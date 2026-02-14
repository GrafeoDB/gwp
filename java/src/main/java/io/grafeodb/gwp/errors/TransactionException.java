package io.grafeodb.gwp.errors;

/**
 * Transaction state error (begin failed, empty transaction ID).
 */
public class TransactionException extends GqlException {

    public TransactionException(String message) {
        super(message);
    }

    public TransactionException(String message, Throwable cause) {
        super(message, cause);
    }
}
