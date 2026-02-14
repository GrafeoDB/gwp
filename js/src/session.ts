/** GQL session management. */

import type {
  SessionServiceClient,
  GqlServiceClient,
} from "./generated/gql_service";
import { promisifyUnary } from "./util";
import { SessionError } from "./errors";
import { ResultCursor } from "./result";
import { Transaction } from "./transaction";
import { valueToProto } from "./convert";
import type { GqlValue } from "./types";

/** An active session with a GWP server. */
export class GqlSession {
  private readonly _sessionId: string;
  private readonly sessionClient: SessionServiceClient;
  private readonly gqlClient: GqlServiceClient;
  private closed = false;

  /** @internal */
  constructor(
    sessionId: string,
    sessionClient: SessionServiceClient,
    gqlClient: GqlServiceClient,
  ) {
    this._sessionId = sessionId;
    this.sessionClient = sessionClient;
    this.gqlClient = gqlClient;
  }

  /** Create a new session via handshake. */
  static async create(
    sessionClient: SessionServiceClient,
    gqlClient: GqlServiceClient,
  ): Promise<GqlSession> {
    const resp = await promisifyUnary(sessionClient, "handshake", {
      protocolVersion: 1,
      credentials: undefined,
      clientInfo: {},
    });

    if (!resp.sessionId) {
      throw new SessionError("server returned empty session ID");
    }

    return new GqlSession(resp.sessionId, sessionClient, gqlClient);
  }

  /** The session identifier. */
  get sessionId(): string {
    return this._sessionId;
  }

  /** Execute a GQL statement. */
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
      sessionId: this._sessionId,
      statement,
      parameters: protoParams,
      transactionId: "",
    });

    return new ResultCursor(stream);
  }

  /** Begin a new transaction. */
  async beginTransaction(options?: {
    readOnly?: boolean;
  }): Promise<Transaction> {
    const mode = options?.readOnly ? 1 : 0; // READ_ONLY = 1, READ_WRITE = 0
    return Transaction.begin(
      this._sessionId,
      this.gqlClient,
      mode,
    );
  }

  /** Set the current graph. */
  async setGraph(name: string): Promise<void> {
    await promisifyUnary(this.sessionClient, "configure", {
      sessionId: this._sessionId,
      graph: name,
    });
  }

  /** Set the current schema. */
  async setSchema(name: string): Promise<void> {
    await promisifyUnary(this.sessionClient, "configure", {
      sessionId: this._sessionId,
      schema: name,
    });
  }

  /** Set the session timezone. */
  async setTimeZone(offsetMinutes: number): Promise<void> {
    await promisifyUnary(this.sessionClient, "configure", {
      sessionId: this._sessionId,
      timeZoneOffsetMinutes: offsetMinutes,
    });
  }

  /** Reset session state to defaults. */
  async reset(): Promise<void> {
    await promisifyUnary(this.sessionClient, "reset", {
      sessionId: this._sessionId,
      target: 0, // RESET_ALL
    });
  }

  /** Ping the server. Returns a timestamp. */
  async ping(): Promise<bigint> {
    const resp = await promisifyUnary(this.sessionClient, "ping", {
      sessionId: this._sessionId,
    });
    return resp.timestamp;
  }

  /** Close the session. */
  async close(): Promise<void> {
    if (!this.closed) {
      await promisifyUnary(this.sessionClient, "close", {
        sessionId: this._sessionId,
      });
      this.closed = true;
    }
  }
}
