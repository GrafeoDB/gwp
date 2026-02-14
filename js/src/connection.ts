/** GQL connection management. */

import { Client, credentials, type ChannelCredentials } from "@grpc/grpc-js";
import { SessionServiceClient, GqlServiceClient } from "./generated/gql_service";
import { GqlSession } from "./session";

/** A connection to a GWP server. */
export class GqlConnection {
  private readonly sessionClient: SessionServiceClient;
  private readonly gqlClient: GqlServiceClient;

  private constructor(
    sessionClient: SessionServiceClient,
    gqlClient: GqlServiceClient,
  ) {
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
    return new GqlConnection(sessionClient, gqlClient);
  }

  /** Perform handshake and return a session. */
  async createSession(): Promise<GqlSession> {
    return GqlSession.create(this.sessionClient, this.gqlClient);
  }

  /** Close the underlying gRPC channels. */
  close(): void {
    Client.prototype.close.call(this.sessionClient);
    Client.prototype.close.call(this.gqlClient);
  }
}
