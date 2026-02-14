package io.grafeodb.gwp.errors;

/**
 * Session-level error (not found, expired, handshake failure).
 */
public class SessionException extends GqlException {

    public SessionException(String message) {
        super(message);
    }

    public SessionException(String message, Throwable cause) {
        super(message, cause);
    }
}
