/** GQL error types. */

export class GqlError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "GqlError";
  }
}

export class ConnectionError extends GqlError {
  constructor(message: string) {
    super(message);
    this.name = "ConnectionError";
  }
}

export class SessionError extends GqlError {
  constructor(message: string) {
    super(message);
    this.name = "SessionError";
  }
}

export class TransactionError extends GqlError {
  constructor(message: string) {
    super(message);
    this.name = "TransactionError";
  }
}

export class GqlStatusError extends GqlError {
  readonly code: string;
  readonly diagnostic?: string;

  constructor(code: string, message: string, diagnostic?: string) {
    super(`[${code}] ${message}`);
    this.name = "GqlStatusError";
    this.code = code;
    this.diagnostic = diagnostic;
  }
}
