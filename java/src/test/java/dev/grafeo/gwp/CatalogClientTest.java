package dev.grafeo.gwp;

import gql.CatalogServiceGrpc;
import gql.GqlServiceGrpc;
import gql.GqlServiceOuterClass;
import gql.SessionServiceGrpc;
import io.grpc.ManagedChannel;
import io.grpc.Server;
import io.grpc.Status;
import io.grpc.StatusRuntimeException;
import io.grpc.inprocess.InProcessChannelBuilder;
import io.grpc.inprocess.InProcessServerBuilder;
import io.grpc.stub.StreamObserver;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.concurrent.atomic.AtomicInteger;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Tests for {@link CatalogClient} session binding.
 *
 * <p>Uses a gRPC in-process server whose catalog service records the session
 * each request names and, like the real server, answers NOT_FOUND for a named
 * session it does not know.</p>
 */
class CatalogClientTest {

    private static final String SERVER_NAME = "gwp-catalog-test-server";

    private Server server;
    private ManagedChannel channel;
    private final AtomicInteger sessionCounter = new AtomicInteger(0);
    private final Set<String> liveSessions = ConcurrentHashMap.newKeySet();
    private final List<String> seenSessionIds = new CopyOnWriteArrayList<>();

    @BeforeEach
    void setUp() throws IOException {
        server = InProcessServerBuilder
                .forName(SERVER_NAME)
                .directExecutor()
                .addService(new MockSessionService())
                .addService(new MockCatalogService())
                .build()
                .start();

        channel = InProcessChannelBuilder
                .forName(SERVER_NAME)
                .directExecutor()
                .build();
    }

    @AfterEach
    void tearDown() {
        channel.shutdownNow();
        server.shutdownNow();
    }

    private GqlSession createSession() {
        return GqlSession.create(
                SessionServiceGrpc.newBlockingStub(channel),
                GqlServiceGrpc.newBlockingStub(channel));
    }

    /** Call every catalog RPC once. */
    private static void callEveryRpc(CatalogClient catalog) {
        catalog.listSchemas();
        catalog.createSchema("s", true);
        catalog.dropSchema("s", true);
        catalog.listGraphs("default");
        catalog.createGraph(new CatalogClient.CreateGraphConfig("default", "g"));
        catalog.dropGraph("default", "g", true);
        catalog.getGraphInfo("default", "default");
        catalog.listGraphTypes("default");
        catalog.createGraphType("default", "t", true, false);
        catalog.dropGraphType("default", "t", true);
    }

    @Test
    void sessionCatalogNamesSessionOnEveryRequest() {
        try (GqlSession session = createSession()) {
            try (CatalogClient catalog = session.catalog()) {
                assertEquals(Optional.of(session.sessionId()), catalog.sessionId());

                callEveryRpc(catalog);

                assertEquals(10, seenSessionIds.size());
                for (String seen : seenSessionIds) {
                    assertEquals(session.sessionId(), seen);
                }
            }
        }
    }

    @Test
    void connectionCatalogNamesNoSession() {
        try (CatalogClient catalog = new CatalogClient(channel)) {
            assertTrue(catalog.sessionId().isEmpty());

            callEveryRpc(catalog);

            assertEquals(10, seenSessionIds.size());
            for (String seen : seenSessionIds) {
                assertEquals("", seen);
            }
        }
    }

    @Test
    void unknownSessionIsNotFound() {
        try (CatalogClient catalog = new CatalogClient(channel, "no-such-session")) {
            assertEquals(Optional.of("no-such-session"), catalog.sessionId());

            StatusRuntimeException e =
                    assertThrows(StatusRuntimeException.class, catalog::listSchemas);
            assertEquals(Status.Code.NOT_FOUND, e.getStatus().getCode());
        }
    }

    @Test
    void closedSessionIsNotFound() {
        GqlSession session = createSession();
        CatalogClient catalog = session.catalog();
        catalog.listSchemas();
        session.close();

        StatusRuntimeException e =
                assertThrows(StatusRuntimeException.class, catalog::listSchemas);
        assertEquals(Status.Code.NOT_FOUND, e.getStatus().getCode());
    }

    // ========================================================================
    // Mock services
    // ========================================================================

    private class MockSessionService extends SessionServiceGrpc.SessionServiceImplBase {

        @Override
        public void handshake(
                GqlServiceOuterClass.HandshakeRequest request,
                StreamObserver<GqlServiceOuterClass.HandshakeResponse> responseObserver) {
            String id = "mock-session-" + sessionCounter.incrementAndGet();
            liveSessions.add(id);
            responseObserver.onNext(GqlServiceOuterClass.HandshakeResponse.newBuilder()
                    .setProtocolVersion(1)
                    .setSessionId(id)
                    .build());
            responseObserver.onCompleted();
        }

        @Override
        public void closeSession(
                GqlServiceOuterClass.CloseSessionRequest request,
                StreamObserver<GqlServiceOuterClass.CloseSessionResponse> responseObserver) {
            liveSessions.remove(request.getSessionId());
            responseObserver.onNext(GqlServiceOuterClass.CloseSessionResponse.getDefaultInstance());
            responseObserver.onCompleted();
        }
    }

    private class MockCatalogService extends CatalogServiceGrpc.CatalogServiceImplBase {

        /**
         * Record the named session and answer with {@code response}, or with
         * NOT_FOUND when the request names a session that is not live.
         */
        private <T> void respond(String sessionId, T response, StreamObserver<T> observer) {
            seenSessionIds.add(sessionId);
            if (!sessionId.isEmpty() && !liveSessions.contains(sessionId)) {
                observer.onError(Status.NOT_FOUND
                        .withDescription("session " + sessionId + " not found")
                        .asRuntimeException());
                return;
            }
            observer.onNext(response);
            observer.onCompleted();
        }

        @Override
        public void listSchemas(
                GqlServiceOuterClass.ListSchemasRequest request,
                StreamObserver<GqlServiceOuterClass.ListSchemasResponse> responseObserver) {
            respond(request.getSessionId(),
                    GqlServiceOuterClass.ListSchemasResponse.getDefaultInstance(), responseObserver);
        }

        @Override
        public void createSchema(
                GqlServiceOuterClass.CreateSchemaRequest request,
                StreamObserver<GqlServiceOuterClass.CreateSchemaResponse> responseObserver) {
            respond(request.getSessionId(),
                    GqlServiceOuterClass.CreateSchemaResponse.getDefaultInstance(), responseObserver);
        }

        @Override
        public void dropSchema(
                GqlServiceOuterClass.DropSchemaRequest request,
                StreamObserver<GqlServiceOuterClass.DropSchemaResponse> responseObserver) {
            respond(request.getSessionId(),
                    GqlServiceOuterClass.DropSchemaResponse.getDefaultInstance(), responseObserver);
        }

        @Override
        public void listGraphs(
                GqlServiceOuterClass.ListGraphsRequest request,
                StreamObserver<GqlServiceOuterClass.ListGraphsResponse> responseObserver) {
            respond(request.getSessionId(),
                    GqlServiceOuterClass.ListGraphsResponse.getDefaultInstance(), responseObserver);
        }

        @Override
        public void createGraph(
                GqlServiceOuterClass.CreateGraphRequest request,
                StreamObserver<GqlServiceOuterClass.CreateGraphResponse> responseObserver) {
            respond(request.getSessionId(),
                    GqlServiceOuterClass.CreateGraphResponse.getDefaultInstance(), responseObserver);
        }

        @Override
        public void dropGraph(
                GqlServiceOuterClass.DropGraphRequest request,
                StreamObserver<GqlServiceOuterClass.DropGraphResponse> responseObserver) {
            respond(request.getSessionId(),
                    GqlServiceOuterClass.DropGraphResponse.getDefaultInstance(), responseObserver);
        }

        @Override
        public void getGraphInfo(
                GqlServiceOuterClass.GetGraphInfoRequest request,
                StreamObserver<GqlServiceOuterClass.GetGraphInfoResponse> responseObserver) {
            respond(request.getSessionId(),
                    GqlServiceOuterClass.GetGraphInfoResponse.getDefaultInstance(), responseObserver);
        }

        @Override
        public void listGraphTypes(
                GqlServiceOuterClass.ListGraphTypesRequest request,
                StreamObserver<GqlServiceOuterClass.ListGraphTypesResponse> responseObserver) {
            respond(request.getSessionId(),
                    GqlServiceOuterClass.ListGraphTypesResponse.getDefaultInstance(), responseObserver);
        }

        @Override
        public void createGraphType(
                GqlServiceOuterClass.CreateGraphTypeRequest request,
                StreamObserver<GqlServiceOuterClass.CreateGraphTypeResponse> responseObserver) {
            respond(request.getSessionId(),
                    GqlServiceOuterClass.CreateGraphTypeResponse.getDefaultInstance(), responseObserver);
        }

        @Override
        public void dropGraphType(
                GqlServiceOuterClass.DropGraphTypeRequest request,
                StreamObserver<GqlServiceOuterClass.DropGraphTypeResponse> responseObserver) {
            respond(request.getSessionId(),
                    GqlServiceOuterClass.DropGraphTypeResponse.getDefaultInstance(), responseObserver);
        }
    }
}
