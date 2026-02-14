/** Transaction management. */

import type { GqlServiceClient } from "./generated/gql_service";
import { promisifyUnary } from "./util";
import { GqlStatusError, TransactionError } from "./errors";
import { ResultCursor } from "./result";
import { isException } from "./status";
import { valueToProto } from "./convert";
import type { GqlValue } from "./types";

/** An explicit transaction within a session. */
export class Transaction {
  private readonly sessionId: string;
  private readonly _transactionId: string;
  private readonly gqlClient: GqlServiceClient;
  private committed = false;
  private rolledBack = false;

  /** @internal */
  constructor(
    sessionId: string,
    transactionId: string,
    gqlClient: GqlServiceClient,
  ) {
    this.sessionId = sessionId;
    this._transactionId = transactionId;
    this.gqlClient = gqlClient;
  }

  /** Begin a new transaction. */
  static async begin(
    sessionId: string,
    gqlClient: GqlServiceClient,
    mode: number,
  ): Promise<Transaction> {
    const resp = await promisifyUnary(gqlClient, "beginTransaction", {
      sessionId,
      mode,
    });

    if (resp.status && isException(resp.status.code)) {
      throw new GqlStatusError(resp.status.code, resp.status.message);
    }

    if (!resp.transactionId) {
      throw new TransactionError("server returned empty transaction ID");
    }

    return new Transaction(sessionId, resp.transactionId, gqlClient);
  }

  /** The transaction identifier. */
  get transactionId(): string {
    return this._transactionId;
  }

  /** Execute a statement within this transaction. */
  async execute(
    statement: string,
    parameters?: Record<string, GqlValue>,
  ): Promise<ResultCursor> {
    const protoParams: Record<string, ReturnType<typeof valueToProto>> = {};
    if (parameters) {
      for (const [k, v] of Object.entries(parameters)) {
        protoParams[k] = valueToProto(v);
      }
    }

    const stream = this.gqlClient.execute({
      sessionId: this.sessionId,
      statement,
      parameters: protoParams,
      transactionId: this._transactionId,
    });

    return new ResultCursor(stream);
  }

  /** Commit the transaction. */
  async commit(): Promise<void> {
    const resp = await promisifyUnary(this.gqlClient, "commit", {
      sessionId: this.sessionId,
      transactionId: this._transactionId,
    });
    this.committed = true;

    if (resp.status && isException(resp.status.code)) {
      throw new GqlStatusError(resp.status.code, resp.status.message);
    }
  }

  /** Roll back the transaction. */
  async rollback(): Promise<void> {
    if (this.committed || this.rolledBack) return;

    const resp = await promisifyUnary(this.gqlClient, "rollback", {
      sessionId: this.sessionId,
      transactionId: this._transactionId,
    });
    this.rolledBack = true;

    if (resp.status && isException(resp.status.code)) {
      throw new GqlStatusError(resp.status.code, resp.status.message);
    }
  }
}
