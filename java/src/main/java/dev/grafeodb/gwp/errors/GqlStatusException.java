package io.grafeodb.gwp.errors;

import io.grafeodb.gwp.GqlStatus;

/**
 * GQL-domain error carrying a GQLSTATUS code and optional diagnostic context.
 */
public class GqlStatusException extends GqlException {

    private final String code;
    private final String gqlMessage;
    private final String diagnosticOperation;
    private final int diagnosticOperationCode;
    private final String diagnosticSchema;

    public GqlStatusException(String code, String message) {
        this(code, message, "", 0, "");
    }

    public GqlStatusException(
            String code,
            String message,
            String diagnosticOperation,
            int diagnosticOperationCode,
            String diagnosticSchema) {
        super("[" + code + "] " + message);
        this.code = code;
        this.gqlMessage = message;
        this.diagnosticOperation = diagnosticOperation;
        this.diagnosticOperationCode = diagnosticOperationCode;
        this.diagnosticSchema = diagnosticSchema;
    }

    /** The 5-character GQLSTATUS code. */
    public String code() {
        return code;
    }

    /** The human-readable GQL status message. */
    public String gqlMessage() {
        return gqlMessage;
    }

    /** The diagnostic operation string, if any. */
    public String diagnosticOperation() {
        return diagnosticOperation;
    }

    /** The diagnostic operation code, if any. */
    public int diagnosticOperationCode() {
        return diagnosticOperationCode;
    }

    /** The diagnostic schema context, if any. */
    public String diagnosticSchema() {
        return diagnosticSchema;
    }

    /** Check if this status represents success. */
    public boolean isSuccess() {
        return GqlStatus.isSuccess(code);
    }

    /** Check if this status represents a warning. */
    public boolean isWarning() {
        return GqlStatus.isWarning(code);
    }

    /** Check if this status represents an exception. */
    public boolean isException() {
        return GqlStatus.isException(code);
    }
}
