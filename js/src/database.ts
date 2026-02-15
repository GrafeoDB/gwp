/** Database management client. */

import type { ChannelCredentials } from "@grpc/grpc-js";
import { DatabaseServiceClient } from "./generated/gql_service";
import { promisifyUnary } from "./util";

/** Summary information about a database. */
export interface DatabaseInfo {
  readonly name: string;
  readonly nodeCount: number;
  readonly edgeCount: number;
  readonly persistent: boolean;
  readonly databaseType: string;
  readonly storageMode?: string;
  readonly memoryLimitBytes?: number;
  readonly backwardEdges?: boolean;
  readonly threads?: number;
}

/** Configuration for creating a new database. */
export interface CreateDatabaseConfig {
  readonly name: string;
  readonly databaseType?: string;
  readonly storageMode?: string;
  readonly memoryLimitBytes?: number;
  readonly backwardEdges?: boolean;
  readonly threads?: number;
  readonly walEnabled?: boolean;
  readonly walDurability?: string;
}

/** A client for managing databases on a GWP server. */
export class DatabaseClient {
  private readonly client: DatabaseServiceClient;

  /** @internal */
  constructor(endpoint: string, creds: ChannelCredentials) {
    this.client = new DatabaseServiceClient(endpoint, creds);
  }

  /** List all databases on the server. */
  async list(): Promise<DatabaseInfo[]> {
    const resp = await promisifyUnary(this.client, "listDatabases", {});
    return resp.databases.map(
      (db: any): DatabaseInfo => ({
        name: db.name,
        nodeCount: Number(db.nodeCount),
        edgeCount: Number(db.edgeCount),
        persistent: db.persistent,
        databaseType: db.databaseType,
      }),
    );
  }

  /** Create a new database. */
  async create(config: CreateDatabaseConfig): Promise<DatabaseInfo> {
    const resp = await promisifyUnary(this.client, "createDatabase", {
      name: config.name,
      databaseType: config.databaseType ?? "Lpg",
      storageMode: config.storageMode ?? "InMemory",
      options: {
        memoryLimitBytes: config.memoryLimitBytes,
        backwardEdges: config.backwardEdges,
        threads: config.threads,
        walEnabled: config.walEnabled,
        walDurability: config.walDurability,
      },
    });
    const db = resp.database;
    return {
      name: db.name,
      nodeCount: Number(db.nodeCount),
      edgeCount: Number(db.edgeCount),
      persistent: db.persistent,
      databaseType: db.databaseType,
    };
  }

  /** Delete a database by name. Returns the deleted database name. */
  async delete(name: string): Promise<string> {
    const resp = await promisifyUnary(this.client, "deleteDatabase", { name });
    return resp.deleted;
  }

  /** Get detailed information about a database. */
  async getInfo(name: string): Promise<DatabaseInfo> {
    const resp = await promisifyUnary(this.client, "getDatabaseInfo", { name });
    return {
      name: resp.name,
      nodeCount: Number(resp.nodeCount),
      edgeCount: Number(resp.edgeCount),
      persistent: resp.persistent,
      databaseType: resp.databaseType,
      storageMode: resp.storageMode || undefined,
      memoryLimitBytes: resp.memoryLimitBytes
        ? Number(resp.memoryLimitBytes)
        : undefined,
      backwardEdges: resp.backwardEdges || undefined,
      threads: resp.threads ? Number(resp.threads) : undefined,
    };
  }

  /** Close the underlying gRPC channel. */
  close(): void {
    this.client.close();
  }
}
