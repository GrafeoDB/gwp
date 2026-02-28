/** Catalog management client (schemas, graphs, graph types). */

import type { ChannelCredentials } from "@grpc/grpc-js";
import { CatalogServiceClient } from "./generated/gql_service";
import { promisifyUnary } from "./util";

/** Summary information about a schema. */
export interface SchemaInfo {
  readonly name: string;
  readonly graphCount: number;
  readonly graphTypeCount: number;
}

/** Summary information about a graph. */
export interface GraphInfo {
  readonly schema: string;
  readonly name: string;
  readonly nodeCount: number;
  readonly edgeCount: number;
  readonly graphType: string;
  readonly storageMode?: string;
  readonly memoryLimitBytes?: number;
  readonly backwardEdges?: boolean;
  readonly threads?: number;
}

/** Summary information about a graph type. */
export interface GraphTypeDetails {
  readonly schema: string;
  readonly name: string;
}

/** Configuration for creating a new graph. */
export interface CreateGraphConfig {
  readonly schema: string;
  readonly name: string;
  readonly ifNotExists?: boolean;
  readonly orReplace?: boolean;
  readonly storageMode?: string;
  readonly memoryLimitBytes?: number;
  readonly backwardEdges?: boolean;
  readonly threads?: number;
  readonly walEnabled?: boolean;
  readonly walDurability?: string;
}

/** A client for managing schemas, graphs, and graph types on a GWP server. */
export class CatalogClient {
  private readonly client: CatalogServiceClient;

  /** @internal */
  constructor(endpoint: string, creds: ChannelCredentials) {
    this.client = new CatalogServiceClient(endpoint, creds);
  }

  // ========================================================================
  // Schema operations
  // ========================================================================

  /** List all schemas on the server. */
  async listSchemas(): Promise<SchemaInfo[]> {
    const resp = await promisifyUnary(this.client, "listSchemas", {});
    return resp.schemas.map(
      (s: any): SchemaInfo => ({
        name: s.name,
        graphCount: Number(s.graphCount),
        graphTypeCount: Number(s.graphTypeCount),
      }),
    );
  }

  /** Create a new schema. */
  async createSchema(name: string, ifNotExists = false): Promise<void> {
    await promisifyUnary(this.client, "createSchema", { name, ifNotExists });
  }

  /** Drop a schema. Returns true if it existed. */
  async dropSchema(name: string, ifExists = false): Promise<boolean> {
    const resp = await promisifyUnary(this.client, "dropSchema", {
      name,
      ifExists,
    });
    return resp.existed;
  }

  // ========================================================================
  // Graph operations
  // ========================================================================

  /** List all graphs in a schema. */
  async listGraphs(schema: string): Promise<GraphInfo[]> {
    const resp = await promisifyUnary(this.client, "listGraphs", { schema });
    return resp.graphs.map(
      (g: any): GraphInfo => ({
        schema: g.schema,
        name: g.name,
        nodeCount: Number(g.nodeCount),
        edgeCount: Number(g.edgeCount),
        graphType: g.graphType,
      }),
    );
  }

  /** Create a new graph. */
  async createGraph(config: CreateGraphConfig): Promise<GraphInfo> {
    const resp = await promisifyUnary(this.client, "createGraph", {
      schema: config.schema,
      name: config.name,
      ifNotExists: config.ifNotExists ?? false,
      orReplace: config.orReplace ?? false,
      storageMode: config.storageMode ?? "InMemory",
      options: {
        memoryLimitBytes:
          config.memoryLimitBytes != null
            ? BigInt(config.memoryLimitBytes)
            : undefined,
        backwardEdges: config.backwardEdges,
        threads: config.threads,
        walEnabled: config.walEnabled,
        walDurability: config.walDurability,
      },
    });
    const g = resp.graph;
    return {
      schema: g.schema,
      name: g.name,
      nodeCount: Number(g.nodeCount),
      edgeCount: Number(g.edgeCount),
      graphType: g.graphType,
    };
  }

  /** Drop a graph. Returns true if it existed. */
  async dropGraph(
    schema: string,
    name: string,
    ifExists = false,
  ): Promise<boolean> {
    const resp = await promisifyUnary(this.client, "dropGraph", {
      schema,
      name,
      ifExists,
    });
    return resp.existed;
  }

  /** Get detailed information about a graph. */
  async getGraphInfo(schema: string, name: string): Promise<GraphInfo> {
    const resp = await promisifyUnary(this.client, "getGraphInfo", {
      schema,
      name,
    });
    return {
      schema: resp.schema,
      name: resp.name,
      nodeCount: Number(resp.nodeCount),
      edgeCount: Number(resp.edgeCount),
      graphType: resp.graphType,
      storageMode: resp.storageMode || undefined,
      memoryLimitBytes: resp.memoryLimitBytes
        ? Number(resp.memoryLimitBytes)
        : undefined,
      backwardEdges: resp.backwardEdges || undefined,
      threads: resp.threads ? Number(resp.threads) : undefined,
    };
  }

  // ========================================================================
  // Graph type operations
  // ========================================================================

  /** List all graph types in a schema. */
  async listGraphTypes(schema: string): Promise<GraphTypeDetails[]> {
    const resp = await promisifyUnary(this.client, "listGraphTypes", {
      schema,
    });
    return resp.graphTypes.map(
      (t: any): GraphTypeDetails => ({
        schema: t.schema,
        name: t.name,
      }),
    );
  }

  /** Create a new graph type. */
  async createGraphType(
    schema: string,
    name: string,
    ifNotExists = false,
    orReplace = false,
  ): Promise<void> {
    await promisifyUnary(this.client, "createGraphType", {
      schema,
      name,
      ifNotExists,
      orReplace,
    });
  }

  /** Drop a graph type. Returns true if it existed. */
  async dropGraphType(
    schema: string,
    name: string,
    ifExists = false,
  ): Promise<boolean> {
    const resp = await promisifyUnary(this.client, "dropGraphType", {
      schema,
      name,
      ifExists,
    });
    return resp.existed;
  }

  /** Close the underlying gRPC channel. */
  close(): void {
    this.client.close();
  }
}
