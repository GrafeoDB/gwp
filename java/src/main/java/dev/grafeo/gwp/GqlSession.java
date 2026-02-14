package dev.grafeodb.gwp;

import dev.grafeodb.gwp.errors.SessionException;
import dev.grafeodb.gwp.internal.ValueConverter;

import java.util.HashMap;
import java.util.Iterator;
import java.util.Map;

/**
 * An active session with a GWP server.
 *
 * <p>Implements {@link AutoCloseable} for use with try-with-resources.</p>
 *
 * <pre>{@code
 * try (GqlConnection conn = GqlConnection.connect("localhost:50051")) {
 *     try (GqlSession session = conn.createSession()) {
 *         ResultCursor cursor = session.execute("MATCH (n) RETURN n");
 *         for (List<Object> row : cursor) {
 *             System.out.println(row);
 *         }
 *     }
 * }
 * }</pre>
 */
public class GqlSession implements AutoCloseable {

    private final String sessionId;
    private final gql.SessionServiceGrpc.SessionServiceBlockingStub sessionStub;
    private final gql.GqlServiceGrpc.GqlServiceBlockingStub gqlStub;
    private boolean closed = false;

    GqlSession(
            String sessionId,
            gql.SessionServiceGrpc.SessionServiceBlockingStub sessionStub,
            gql.GqlServiceGrpc.GqlServiceBlockingStub gqlStub) {
        this.sessionId = sessionId;
        this.sessionStub = sessionStub;
        this.gqlStub = gqlStub;
    }

    /**
     * Create a new session by performing a handshake with the server.
     *
     * @param sessionStub the session service stub
     * @param gqlStub     the GQL service stub
     * @return the new session
     * @throws SessionException if the handshake fails or returns an empty session ID
     */
    static GqlSession create(
            gql.SessionServiceGrpc.SessionServiceBlockingStub sessionStub,
            gql.GqlServiceGrpc.GqlServiceBlockingStub gqlStub) {

        gql.GqlService.HandshakeResponse resp = sessionStub.handshake(
                gql.GqlService.HandshakeRequest.newBuilder()
                        .setProtocolVersion(1)
                        .build());

        if (resp.getSessionId().isEmpty()) {
            throw new SessionException("server returned empty session ID");
        }

        return new GqlSession(resp.getSessionId(), sessionStub, gqlStub);
    }

    /** The session identifier. */
    public String sessionId() {
        return sessionId;
    }

    // ========================================================================
    // Query execution
    // ========================================================================

    /**
     * Execute a GQL statement.
     *
     * @param statement the GQL statement to execute
     * @return a cursor over the results
     */
    public ResultCursor execute(String statement) {
        return execute(statement, null);
    }

    /**
     * Execute a GQL statement with parameters.
     *
     * @param statement  the GQL statement to execute
     * @param parameters named parameters (may be null)
     * @return a cursor over the results
     */
    public ResultCursor execute(String statement, Map<String, Object> parameters) {
        gql.GqlService.ExecuteRequest.Builder reqBuilder =
                gql.GqlService.ExecuteRequest.newBuilder()
                        .setSessionId(sessionId)
                        .setStatement(statement);

        if (parameters != null && !parameters.isEmpty()) {
            Map<String, gql.GqlTypes.Value> protoParams = new HashMap<>(parameters.size());
            for (Map.Entry<String, Object> entry : parameters.entrySet()) {
                protoParams.put(entry.getKey(), ValueConverter.toProto(entry.getValue()));
            }
            reqBuilder.putAllParameters(protoParams);
        }

        Iterator<gql.GqlService.ExecuteResponse> stream =
                gqlStub.execute(reqBuilder.build());

        return new ResultCursor(stream);
    }

    // ========================================================================
    // Transactions
    // ========================================================================

    /**
     * Begin a new read-write transaction.
     *
     * @return the new transaction
     */
    public Transaction beginTransaction() {
        return beginTransaction(false);
    }

    /**
     * Begin a new transaction.
     *
     * @param readOnly true for a read-only transaction
     * @return the new transaction
     */
    public Transaction beginTransaction(boolean readOnly) {
        return Transaction.begin(sessionId, gqlStub, sessionStub, readOnly);
    }

    // ========================================================================
    // Session configuration
    // ========================================================================

    /**
     * Set the current graph for this session.
     *
     * @param name the graph name
     */
    public void setGraph(String name) {
        sessionStub.configure(
                gql.GqlService.ConfigureRequest.newBuilder()
                        .setSessionId(sessionId)
                        .setGraph(name)
                        .build());
    }

    /**
     * Set the current schema for this session.
     *
     * @param name the schema name
     */
    public void setSchema(String name) {
        sessionStub.configure(
                gql.GqlService.ConfigureRequest.newBuilder()
                        .setSessionId(sessionId)
                        .setSchema(name)
                        .build());
    }

    /**
     * Set the session timezone as a UTC offset in minutes.
     *
     * @param offsetMinutes the UTC offset in minutes (e.g. 60 for UTC+1)
     */
    public void setTimeZone(int offsetMinutes) {
        sessionStub.configure(
                gql.GqlService.ConfigureRequest.newBuilder()
                        .setSessionId(sessionId)
                        .setTimeZoneOffsetMinutes(offsetMinutes)
                        .build());
    }

    // ========================================================================
    // Session lifecycle
    // ========================================================================

    /**
     * Reset session state to defaults.
     */
    public void reset() {
        sessionStub.reset(
                gql.GqlService.ResetRequest.newBuilder()
                        .setSessionId(sessionId)
                        .setTarget(gql.GqlService.ResetTarget.RESET_ALL)
                        .build());
    }

    /**
     * Ping the server. Returns a server timestamp.
     *
     * @return the server timestamp
     */
    public long ping() {
        gql.GqlService.PongResponse resp = sessionStub.ping(
                gql.GqlService.PingRequest.newBuilder()
                        .setSessionId(sessionId)
                        .build());
        return resp.getTimestamp();
    }

    /**
     * Close the session. Rolls back any active transaction on the server side.
     */
    @Override
    public void close() {
        if (!closed) {
            try {
                sessionStub.close(
                        gql.GqlService.CloseRequest.newBuilder()
                                .setSessionId(sessionId)
                                .build());
            } catch (Exception e) {
                // Suppress errors during close
            }
            closed = true;
        }
    }
}
