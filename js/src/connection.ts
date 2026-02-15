/** GQL connection management. */

import { Client, credentials, type ChannelCredentials } from "@grpc/grpc-js";
import { SessionServiceClient, GqlServiceClient } from "./generated/gql_service";
import { DatabaseClient } from "./database";
import { GqlSession } from "./session";

/** A connection to a GWP server. */
export class GqlConnection {
  private readonly sessionClient: SessionServiceClient;
  private readonly gqlClient: GqlServiceClient;
  private readonly _endpoint: string;
  private readonly _creds: ChannelCredentials;

  private constructor(
    endpoint: string,
    creds: ChannelCredentials,
    sessionClient: SessionServiceClient,
    gqlClient: GqlServiceClient,
  ) {
    this._endpoint = endpoint;
    this._creds = creds;
    this.sessionClient = sessionClient;
    this.gqlClient = gqlClient;
  }

  /**
   * Connect to a GWP server.
   * @param endpoint Server address (e.g. "localhost:50051").
   * @param credentials Optional channel credentials. Defaults to insecure.
   */
  static connect(
    endpoint: string,
    channelCredentials?: ChannelCredentials,
  ): GqlConnection {
    const creds = channelCredentials ?? credentials.createInsecure();
    const sessionClient = new SessionServiceClient(endpoint, creds);
    const gqlClient = new GqlServiceClient(endpoint, creds);
    return new GqlConnection(endpoint, creds, sessionClient, gqlClient);
  }

  /** Perform handshake and return a session. */
  async createSession(): Promise<GqlSession> {
    return GqlSession.create(this.sessionClient, this.gqlClient);
  }

  /** Create a database management client. */
  createDatabaseClient(): DatabaseClient {
    return new DatabaseClient(this._endpoint, this._creds);
  }

  /** Close the underlying gRPC channels. */
  close(): void {
    Client.prototype.close.call(this.sessionClient);
    Client.prototype.close.call(this.gqlClient);
  }
}
