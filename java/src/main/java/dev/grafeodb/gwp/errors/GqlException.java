package io.grafeodb.gwp.errors;

/**
 * Base exception for all GWP errors.
 */
public class GqlException extends RuntimeException {

    public GqlException(String message) {
        super(message);
    }

    public GqlException(String message, Throwable cause) {
        super(message, cause);
    }
}
