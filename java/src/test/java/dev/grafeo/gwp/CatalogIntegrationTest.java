package dev.grafeo.gwp;

import io.grpc.Grpc;
import io.grpc.InsecureChannelCredentials;
import io.grpc.ManagedChannel;
import io.grpc.Status;
import io.grpc.StatusRuntimeException;
import org.junit.jupiter.api.AfterAll;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.net.InetSocketAddress;
import java.net.ServerSocket;
import java.net.Socket;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.TimeUnit;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assumptions.assumeTrue;

/**
 * Catalog tests against the real {@code gwp-test-server}.
 *
 * <p>Runs only when the server binary has been built at
 * {@code ../target/release/gwp-test-server}; otherwise every test is skipped.</p>
 */
class CatalogIntegrationTest {

    private static Process serverProcess;
    private static String endpoint;

    private GqlConnection conn;

    @BeforeAll
    static void startServer() throws Exception {
        String name = System.getProperty("os.name").toLowerCase().contains("win")
                ? "gwp-test-server.exe"
                : "gwp-test-server";
        Path binary = Path.of("..", "target", "release", name).toAbsolutePath().normalize();
        assumeTrue(Files.isExecutable(binary), "gwp-test-server not found at " + binary);

        int port;
        try (ServerSocket socket = new ServerSocket(0)) {
            port = socket.getLocalPort();
        }

        serverProcess = new ProcessBuilder(binary.toString(), Integer.toString(port))
                .redirectErrorStream(true)
                .redirectOutput(ProcessBuilder.Redirect.DISCARD)
                .start();

        long deadline = System.nanoTime() + TimeUnit.SECONDS.toNanos(10);
        while (true) {
            try (Socket probe = new Socket()) {
                probe.connect(new InetSocketAddress("127.0.0.1", port), 500);
                break;
            } catch (IOException e) {
                if (System.nanoTime() > deadline || !serverProcess.isAlive()) {
                    throw new IllegalStateException("gwp-test-server did not start", e);
                }
                Thread.sleep(100);
            }
        }
        endpoint = "localhost:" + port;
    }

    @AfterAll
    static void stopServer() throws InterruptedException {
        if (serverProcess != null) {
            serverProcess.destroy();
            if (!serverProcess.waitFor(5, TimeUnit.SECONDS)) {
                serverProcess.destroyForcibly();
            }
        }
    }

    @BeforeEach
    void connect() {
        conn = GqlConnection.connect(endpoint);
    }

    @AfterEach
    void disconnect() {
        conn.close();
    }

    @Test
    void sessionCatalogListsSchemas() {
        try (GqlSession session = conn.createSession()) {
            try (CatalogClient catalog = session.catalog()) {
                assertEquals(Optional.of(session.sessionId()), catalog.sessionId());

                List<CatalogClient.SchemaDetails> schemas = catalog.listSchemas();
                assertTrue(schemas.stream().anyMatch(s -> s.name().equals("default")));
                assertTrue(catalog.listGraphs("default").stream()
                        .anyMatch(g -> g.name().equals("default")));
            }
        }
    }

    @Test
    void connectionCatalogListsSchemas() {
        try (CatalogClient catalog = conn.createCatalogClient()) {
            assertTrue(catalog.sessionId().isEmpty());

            List<CatalogClient.SchemaDetails> schemas = catalog.listSchemas();
            assertTrue(schemas.stream().anyMatch(s -> s.name().equals("default")));
        }
    }

    @Test
    void unknownSessionIsNotFound() {
        ManagedChannel channel = Grpc.newChannelBuilder(
                endpoint, InsecureChannelCredentials.create()).build();
        try (CatalogClient catalog = new CatalogClient(channel, "no-such-session")) {
            assertEquals(Optional.of("no-such-session"), catalog.sessionId());

            StatusRuntimeException e =
                    assertThrows(StatusRuntimeException.class, catalog::listSchemas);
            assertEquals(Status.Code.NOT_FOUND, e.getStatus().getCode());
        } finally {
            channel.shutdownNow();
        }
    }

    @Test
    void closedSessionIsNotFound() {
        GqlSession session = conn.createSession();
        CatalogClient catalog = session.catalog();
        catalog.listSchemas();
        session.close();

        StatusRuntimeException e =
                assertThrows(StatusRuntimeException.class, catalog::listSchemas);
        assertEquals(Status.Code.NOT_FOUND, e.getStatus().getCode());
    }
}
