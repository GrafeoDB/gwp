package dev.grafeo.gwp;

import dev.grafeo.gwp.errors.GqlException;
import io.grpc.ChannelCredentials;
import io.grpc.Grpc;
import io.grpc.InsecureChannelCredentials;
import io.grpc.ManagedChannel;
import io.grpc.TlsChannelCredentials;

import java.util.concurrent.TimeUnit;

/**
 * A connection to a GWP server.
 *
 * <p>Implements {@link AutoCloseable} for use with try-with-resources.</p>
 *
 * <pre>{@code
 * try (GqlConnection conn = GqlConnection.connect("localhost:50051")) {
 *     try (GqlSession session = conn.createSession()) {
 *         // use the session
 *     }
 * }
 * }</pre>
 */
public class GqlConnection implements AutoCloseable {

    private final ManagedChannel channel;

    private GqlConnection(ManagedChannel channel) {
        this.channel = channel;
    }

    /**
     * Connect to a GWP server using an insecure (plaintext) channel.
     *
     * @param target the server address (e.g. "localhost:50051")
     * @return a connected GqlConnection
     * @throws GqlException if the connection fails
     */
    public static GqlConnection connect(String target) {
        return connect(target, false);
    }

    /**
     * Connect to a GWP server.
     *
     * @param target the server address (e.g. "localhost:50051")
     * @param useTls true to use TLS, false for plaintext
     * @return a connected GqlConnection
     * @throws GqlException if the connection fails
     */
    public static GqlConnection connect(String target, boolean useTls) {
        try {
            ChannelCredentials credentials = useTls
                    ? TlsChannelCredentials.create()
                    : InsecureChannelCredentials.create();

            ManagedChannel channel = Grpc.newChannelBuilder(target, credentials)
                    .build();

            return new GqlConnection(channel);
        } catch (Exception e) {
            throw new GqlException("failed to connect to " + target + ": " + e.getMessage(), e);
        }
    }

    /**
     * Perform a handshake and return a new session.
     *
     * @return a new GqlSession
     */
    public GqlSession createSession() {
        gql.SessionServiceGrpc.SessionServiceBlockingStub sessionStub =
                gql.SessionServiceGrpc.newBlockingStub(channel);
        gql.GqlServiceGrpc.GqlServiceBlockingStub gqlStub =
                gql.GqlServiceGrpc.newBlockingStub(channel);

        return GqlSession.create(sessionStub, gqlStub);
    }

    /**
     * Close the underlying gRPC channel.
     *
     * <p>Initiates an orderly shutdown and waits up to 5 seconds for
     * in-progress RPCs to complete.</p>
     */
    @Override
    public void close() {
        channel.shutdown();
        try {
            if (!channel.awaitTermination(5, TimeUnit.SECONDS)) {
                channel.shutdownNow();
            }
        } catch (InterruptedException e) {
            channel.shutdownNow();
            Thread.currentThread().interrupt();
        }
    }
}
