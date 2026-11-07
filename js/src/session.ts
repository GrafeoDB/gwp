/** GQL session management. */

import type { ChannelCredentials } from "@grpc/grpc-js";
import type {
  SessionServiceClient,
  GqlServiceClient,
} from "./generated/gql_service";
import { CatalogClient } from "./catalog";
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
  private readonly endpoint: string;
  private readonly creds: ChannelCredentials;
  private closed = false;

  /** @internal */
  constructor(
    sessionId: string,
    sessionClient: SessionServiceClient,
    gqlClient: GqlServiceClient,
    endpoint: string,
    creds: ChannelCredentials,
  ) {
    this._sessionId = sessionId;
    this.sessionClient = sessionClient;
    this.gqlClient = gqlClient;
    this.endpoint = endpoint;
    this.creds = creds;
  }

  /** Create a new session via handshake. */
  static async create(
    sessionClient: SessionServiceClient,
    gqlClient: GqlServiceClient,
    endpoint: string,
    creds: ChannelCredentials,
  ): Promise<GqlSession> {
    const resp = await promisifyUnary(sessionClient, "handshake", {
      protocolVersion: 1,
      credentials: undefined,
      clientInfo: {},
    });

    if (!resp.sessionId) {
      throw new SessionError("server returned empty session ID");
    }

    return new GqlSession(
      resp.sessionId,
      sessionClient,
      gqlClient,
      endpoint,
      creds,
    );
  }

  /** The session identifier. */
  get sessionId(): string {
    return this._sessionId;
  }

  /**
   * A catalog client that works in this session's catalog.
   *
   * Its requests name this session, so a server that keeps one catalog per
   * session shows this session's schemas and graphs;
   * `GqlConnection.createCatalogClient()` gives one that names no session.
   * The client shares this session's channel. This binding sends no
   * handshake credentials yet, so it cannot open a session on a server with
   * authentication.
   */
  catalog(): CatalogClient {
    return new CatalogClient(
      this.endpoint,
      this.creds,
      this._sessionId,
      this.sessionClient.getChannel(),
    );
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
      await promisifyUnary(this.sessionClient, "closeSession", {
        sessionId: this._sessionId,
      });
      this.closed = true;
    }
  }
}
