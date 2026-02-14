package dev.grafeo.gwp;

import dev.grafeo.gwp.errors.GqlStatusException;
import dev.grafeo.gwp.errors.TransactionException;
import dev.grafeo.gwp.internal.ValueConverter;

import java.util.HashMap;
import java.util.Iterator;
import java.util.Map;

/**
 * An explicit transaction within a session.
 *
 * <p>Implements {@link AutoCloseable} for use with try-with-resources. If the
 * transaction has not been committed when {@link #close()} is called, it will
 * be automatically rolled back.</p>
 *
 * <pre>{@code
 * try (Transaction tx = session.beginTransaction()) {
 *     tx.execute("INSERT (:Person {name: 'Alice'})");
 *     tx.commit();
 * }
 * // If commit() was not called, rollback happens automatically
 * }</pre>
 */
public class Transaction implements AutoCloseable {

    private final String sessionId;
    private final String transactionId;
    private final gql.GqlServiceGrpc.GqlServiceBlockingStub gqlStub;
    private final gql.SessionServiceGrpc.SessionServiceBlockingStub sessionStub;
    private boolean committed = false;
    private boolean rolledBack = false;

    Transaction(
            String sessionId,
            String transactionId,
            gql.GqlServiceGrpc.GqlServiceBlockingStub gqlStub,
            gql.SessionServiceGrpc.SessionServiceBlockingStub sessionStub) {
        this.sessionId = sessionId;
        this.transactionId = transactionId;
        this.gqlStub = gqlStub;
        this.sessionStub = sessionStub;
    }

    /**
     * Begin a new transaction.
     *
     * @param sessionId   the session ID
     * @param gqlStub     the GQL service stub
     * @param sessionStub the session service stub
     * @param readOnly    true for a read-only transaction
     * @return the new transaction
     * @throws GqlStatusException  if the server returns an exception status
     * @throws TransactionException if the server returns an empty transaction ID
     */
    static Transaction begin(
            String sessionId,
            gql.GqlServiceGrpc.GqlServiceBlockingStub gqlStub,
            gql.SessionServiceGrpc.SessionServiceBlockingStub sessionStub,
            boolean readOnly) {

        gql.GqlServiceOuterClass.TransactionMode mode = readOnly
                ? gql.GqlServiceOuterClass.TransactionMode.READ_ONLY
                : gql.GqlServiceOuterClass.TransactionMode.READ_WRITE;

        gql.GqlServiceOuterClass.BeginResponse resp = gqlStub.beginTransaction(
                gql.GqlServiceOuterClass.BeginRequest.newBuilder()
                        .setSessionId(sessionId)
                        .setMode(mode)
                        .build());

        if (resp.hasStatus() && GqlStatus.isException(resp.getStatus().getCode())) {
            throw new GqlStatusException(
                    resp.getStatus().getCode(),
                    resp.getStatus().getMessage());
        }

        if (resp.getTransactionId().isEmpty()) {
            throw new TransactionException("server returned empty transaction ID");
        }

        return new Transaction(sessionId, resp.getTransactionId(), gqlStub, sessionStub);
    }

    /** The transaction identifier. */
    public String transactionId() {
        return transactionId;
    }

    /**
     * Execute a GQL statement within this transaction.
     *
     * @param statement the GQL statement to execute
     * @return a cursor over the results
     */
    public ResultCursor execute(String statement) {
        return execute(statement, null);
    }

    /**
     * Execute a GQL statement within this transaction with parameters.
     *
     * @param statement  the GQL statement to execute
     * @param parameters named parameters (may be null)
     * @return a cursor over the results
     */
    public ResultCursor execute(String statement, Map<String, Object> parameters) {
        gql.GqlServiceOuterClass.ExecuteRequest.Builder reqBuilder =
                gql.GqlServiceOuterClass.ExecuteRequest.newBuilder()
                        .setSessionId(sessionId)
                        .setStatement(statement)
                        .setTransactionId(transactionId);

        if (parameters != null && !parameters.isEmpty()) {
            Map<String, gql.GqlTypes.Value> protoParams = new HashMap<>(parameters.size());
            for (Map.Entry<String, Object> entry : parameters.entrySet()) {
                protoParams.put(entry.getKey(), ValueConverter.toProto(entry.getValue()));
            }
            reqBuilder.putAllParameters(protoParams);
        }

        Iterator<gql.GqlServiceOuterClass.ExecuteResponse> stream =
                gqlStub.execute(reqBuilder.build());

        return new ResultCursor(stream);
    }

    /**
     * Commit the transaction.
     *
     * @throws GqlStatusException if the server returns an exception status
     */
    public void commit() {
        gql.GqlServiceOuterClass.CommitResponse resp = gqlStub.commit(
                gql.GqlServiceOuterClass.CommitRequest.newBuilder()
                        .setSessionId(sessionId)
                        .setTransactionId(transactionId)
                        .build());

        committed = true;

        if (resp.hasStatus() && GqlStatus.isException(resp.getStatus().getCode())) {
            throw new GqlStatusException(
                    resp.getStatus().getCode(),
                    resp.getStatus().getMessage());
        }
    }

    /**
     * Roll back the transaction.
     *
     * <p>This is a no-op if the transaction has already been committed or
     * rolled back.</p>
     *
     * @throws GqlStatusException if the server returns an exception status
     */
    public void rollback() {
        if (committed || rolledBack) {
            return;
        }

        gql.GqlServiceOuterClass.RollbackResponse resp = gqlStub.rollback(
                gql.GqlServiceOuterClass.RollbackRequest.newBuilder()
                        .setSessionId(sessionId)
                        .setTransactionId(transactionId)
                        .build());

        rolledBack = true;

        if (resp.hasStatus() && GqlStatus.isException(resp.getStatus().getCode())) {
            throw new GqlStatusException(
                    resp.getStatus().getCode(),
                    resp.getStatus().getMessage());
        }
    }

    /**
     * Close the transaction. If the transaction has not been committed,
     * it will be rolled back automatically.
     */
    @Override
    public void close() {
        if (!committed && !rolledBack) {
            try {
                rollback();
            } catch (Exception e) {
                // Suppress rollback errors during close
            }
        }
    }
}
