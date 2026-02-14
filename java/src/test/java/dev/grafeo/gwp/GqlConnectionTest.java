package dev.grafeodb.gwp;

import gql.GqlService;
import gql.GqlServiceGrpc;
import gql.GqlTypes;
import gql.SessionServiceGrpc;
import io.grpc.ManagedChannel;
import io.grpc.Server;
import io.grpc.inprocess.InProcessChannelBuilder;
import io.grpc.inprocess.InProcessServerBuilder;
import io.grpc.stub.StreamObserver;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.util.List;
import java.util.concurrent.atomic.AtomicInteger;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Integration tests for {@link GqlConnection}, {@link GqlSession},
 * {@link ResultCursor}, and {@link Transaction}.
 *
 * <p>Uses a gRPC in-process server with mock service implementations.</p>
 */
class GqlConnectionTest {

    private static final String SERVER_NAME = "gwp-test-server";

    private Server server;
    private ManagedChannel channel;
    private final AtomicInteger sessionCounter = new AtomicInteger(0);

    @BeforeEach
    void setUp() throws IOException {
        server = InProcessServerBuilder
                .forName(SERVER_NAME)
                .directExecutor()
                .addService(new MockSessionService())
                .addService(new MockGqlService())
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

    // ========================================================================
    // Connection and session lifecycle
    // ========================================================================

    @Test
    void connectAndCreateSession() {
        SessionServiceGrpc.SessionServiceBlockingStub sessionStub =
                SessionServiceGrpc.newBlockingStub(channel);
        GqlServiceGrpc.GqlServiceBlockingStub gqlStub =
                GqlServiceGrpc.newBlockingStub(channel);

        GqlSession session = GqlSession.create(sessionStub, gqlStub);
        assertNotNull(session.sessionId());
        assertTrue(session.sessionId().startsWith("mock-session-"));
        session.close();
    }

    @Test
    void sessionTryWithResources() {
        SessionServiceGrpc.SessionServiceBlockingStub sessionStub =
                SessionServiceGrpc.newBlockingStub(channel);
        GqlServiceGrpc.GqlServiceBlockingStub gqlStub =
                GqlServiceGrpc.newBlockingStub(channel);

        try (GqlSession session = GqlSession.create(sessionStub, gqlStub)) {
            assertNotNull(session.sessionId());
        }
    }

    // ========================================================================
    // Ping
    // ========================================================================

    @Test
    void ping() {
        SessionServiceGrpc.SessionServiceBlockingStub sessionStub =
                SessionServiceGrpc.newBlockingStub(channel);
        GqlServiceGrpc.GqlServiceBlockingStub gqlStub =
                GqlServiceGrpc.newBlockingStub(channel);

        try (GqlSession session = GqlSession.create(sessionStub, gqlStub)) {
            long ts = session.ping();
            assertTrue(ts > 0);
        }
    }

    // ========================================================================
    // Execute
    // ========================================================================

    @Test
    void executeAndCollectRows() {
        SessionServiceGrpc.SessionServiceBlockingStub sessionStub =
                SessionServiceGrpc.newBlockingStub(channel);
        GqlServiceGrpc.GqlServiceBlockingStub gqlStub =
                GqlServiceGrpc.newBlockingStub(channel);

        try (GqlSession session = GqlSession.create(sessionStub, gqlStub)) {
            try (ResultCursor cursor = session.execute("MATCH (n) RETURN n.name")) {
                List<String> columns = cursor.columnNames();
                assertEquals(List.of("n.name"), columns);

                List<List<Object>> rows = cursor.collectRows();
                assertEquals(2, rows.size());
                assertEquals("Alice", rows.get(0).get(0));
                assertEquals("Bob", rows.get(1).get(0));

                assertTrue(cursor.isSuccess());
            }
        }
    }

    @Test
    void executeIterator() {
        SessionServiceGrpc.SessionServiceBlockingStub sessionStub =
                SessionServiceGrpc.newBlockingStub(channel);
        GqlServiceGrpc.GqlServiceBlockingStub gqlStub =
                GqlServiceGrpc.newBlockingStub(channel);

        try (GqlSession session = GqlSession.create(sessionStub, gqlStub)) {
            try (ResultCursor cursor = session.execute("MATCH (n) RETURN n.name")) {
                int count = 0;
                while (cursor.hasNext()) {
                    List<Object> row = cursor.next();
                    assertNotNull(row);
                    assertFalse(row.isEmpty());
                    count++;
                }
                assertEquals(2, count);
            }
        }
    }

    // ========================================================================
    // Transaction
    // ========================================================================

    @Test
    void transactionCommit() {
        SessionServiceGrpc.SessionServiceBlockingStub sessionStub =
                SessionServiceGrpc.newBlockingStub(channel);
        GqlServiceGrpc.GqlServiceBlockingStub gqlStub =
                GqlServiceGrpc.newBlockingStub(channel);

        try (GqlSession session = GqlSession.create(sessionStub, gqlStub)) {
            try (Transaction tx = session.beginTransaction()) {
                assertNotNull(tx.transactionId());
                assertTrue(tx.transactionId().startsWith("mock-tx-"));

                try (ResultCursor cursor = tx.execute("CREATE (:Person {name: 'Eve'})")) {
                    cursor.collectRows();
                    assertTrue(cursor.isSuccess());
                }

                tx.commit();
            }
        }
    }

    @Test
    void transactionAutoRollbackOnClose() {
        SessionServiceGrpc.SessionServiceBlockingStub sessionStub =
                SessionServiceGrpc.newBlockingStub(channel);
        GqlServiceGrpc.GqlServiceBlockingStub gqlStub =
                GqlServiceGrpc.newBlockingStub(channel);

        try (GqlSession session = GqlSession.create(sessionStub, gqlStub)) {
            try (Transaction tx = session.beginTransaction()) {
                tx.execute("CREATE (:Person {name: 'Eve'})").close();
                // Not calling commit() - should auto-rollback on close
            }
            // No exception means rollback succeeded
        }
    }

    @Test
    void readOnlyTransaction() {
        SessionServiceGrpc.SessionServiceBlockingStub sessionStub =
                SessionServiceGrpc.newBlockingStub(channel);
        GqlServiceGrpc.GqlServiceBlockingStub gqlStub =
                GqlServiceGrpc.newBlockingStub(channel);

        try (GqlSession session = GqlSession.create(sessionStub, gqlStub)) {
            try (Transaction tx = session.beginTransaction(true)) {
                assertNotNull(tx.transactionId());
                tx.commit();
            }
        }
    }

    // ========================================================================
    // Session configuration
    // ========================================================================

    @Test
    void setGraphAndSchema() {
        SessionServiceGrpc.SessionServiceBlockingStub sessionStub =
                SessionServiceGrpc.newBlockingStub(channel);
        GqlServiceGrpc.GqlServiceBlockingStub gqlStub =
                GqlServiceGrpc.newBlockingStub(channel);

        try (GqlSession session = GqlSession.create(sessionStub, gqlStub)) {
            // These should not throw
            session.setGraph("my_graph");
            session.setSchema("my_schema");
            session.setTimeZone(60);
            session.reset();
        }
    }

    // ========================================================================
    // Mock services
    // ========================================================================

    private class MockSessionService extends SessionServiceGrpc.SessionServiceImplBase {

        @Override
        public void handshake(
                GqlService.HandshakeRequest request,
                StreamObserver<GqlService.HandshakeResponse> responseObserver) {
            responseObserver.onNext(GqlService.HandshakeResponse.newBuilder()
                    .setProtocolVersion(1)
                    .setSessionId("mock-session-" + sessionCounter.incrementAndGet())
                    .setServerInfo(GqlService.ServerInfo.newBuilder()
                            .setName("mock-server")
                            .setVersion("0.1.0")
                            .build())
                    .build());
            responseObserver.onCompleted();
        }

        @Override
        public void configure(
                GqlService.ConfigureRequest request,
                StreamObserver<GqlService.ConfigureResponse> responseObserver) {
            responseObserver.onNext(GqlService.ConfigureResponse.getDefaultInstance());
            responseObserver.onCompleted();
        }

        @Override
        public void reset(
                GqlService.ResetRequest request,
                StreamObserver<GqlService.ResetResponse> responseObserver) {
            responseObserver.onNext(GqlService.ResetResponse.getDefaultInstance());
            responseObserver.onCompleted();
        }

        @Override
        public void close(
                GqlService.CloseRequest request,
                StreamObserver<GqlService.CloseResponse> responseObserver) {
            responseObserver.onNext(GqlService.CloseResponse.getDefaultInstance());
            responseObserver.onCompleted();
        }

        @Override
        public void ping(
                GqlService.PingRequest request,
                StreamObserver<GqlService.PongResponse> responseObserver) {
            responseObserver.onNext(GqlService.PongResponse.newBuilder()
                    .setTimestamp(System.currentTimeMillis())
                    .build());
            responseObserver.onCompleted();
        }
    }

    private static class MockGqlService extends GqlServiceGrpc.GqlServiceImplBase {

        private static final AtomicInteger txCounter = new AtomicInteger(0);

        @Override
        public void execute(
                GqlService.ExecuteRequest request,
                StreamObserver<GqlService.ExecuteResponse> responseObserver) {

            // Send header
            responseObserver.onNext(GqlService.ExecuteResponse.newBuilder()
                    .setHeader(GqlService.ResultHeader.newBuilder()
                            .setResultType(GqlService.ResultType.BINDING_TABLE)
                            .addColumns(GqlService.ColumnDescriptor.newBuilder()
                                    .setName("n.name")
                                    .build())
                            .build())
                    .build());

            // Send a batch of rows
            responseObserver.onNext(GqlService.ExecuteResponse.newBuilder()
                    .setRowBatch(GqlService.RowBatch.newBuilder()
                            .addRows(GqlService.Row.newBuilder()
                                    .addValues(GqlTypes.Value.newBuilder()
                                            .setStringValue("Alice")
                                            .build())
                                    .build())
                            .addRows(GqlService.Row.newBuilder()
                                    .addValues(GqlTypes.Value.newBuilder()
                                            .setStringValue("Bob")
                                            .build())
                                    .build())
                            .build())
                    .build());

            // Send summary
            responseObserver.onNext(GqlService.ExecuteResponse.newBuilder()
                    .setSummary(GqlService.ResultSummary.newBuilder()
                            .setStatus(GqlTypes.GqlStatus.newBuilder()
                                    .setCode("00000")
                                    .setMessage("Success")
                                    .build())
                            .setRowsAffected(2)
                            .build())
                    .build());

            responseObserver.onCompleted();
        }

        @Override
        public void beginTransaction(
                GqlService.BeginRequest request,
                StreamObserver<GqlService.BeginResponse> responseObserver) {
            responseObserver.onNext(GqlService.BeginResponse.newBuilder()
                    .setTransactionId("mock-tx-" + txCounter.incrementAndGet())
                    .setStatus(GqlTypes.GqlStatus.newBuilder()
                            .setCode("00000")
                            .setMessage("Transaction started")
                            .build())
                    .build());
            responseObserver.onCompleted();
        }

        @Override
        public void commit(
                GqlService.CommitRequest request,
                StreamObserver<GqlService.CommitResponse> responseObserver) {
            responseObserver.onNext(GqlService.CommitResponse.newBuilder()
                    .setStatus(GqlTypes.GqlStatus.newBuilder()
                            .setCode("00000")
                            .setMessage("Committed")
                            .build())
                    .build());
            responseObserver.onCompleted();
        }

        @Override
        public void rollback(
                GqlService.RollbackRequest request,
                StreamObserver<GqlService.RollbackResponse> responseObserver) {
            responseObserver.onNext(GqlService.RollbackResponse.newBuilder()
                    .setStatus(GqlTypes.GqlStatus.newBuilder()
                            .setCode("00000")
                            .setMessage("Rolled back")
                            .build())
                    .build());
            responseObserver.onCompleted();
        }
    }
}
