import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { credentials, status, type ServiceError } from "@grpc/grpc-js";
import { CatalogClient } from "../src/catalog";
import { GqlConnection } from "../src/connection";
import { GqlSession } from "../src/session";
import { getTestServer, stopTestServer } from "./helpers";

let endpoint: string;
let conn: GqlConnection;
let session: GqlSession;

beforeAll(async () => {
  endpoint = await getTestServer();
  conn = GqlConnection.connect(endpoint);
  session = await conn.createSession();
});

afterAll(async () => {
  await session.close();
  conn.close();
  stopTestServer();
});

describe("catalog", () => {
  it("session-bound client names its session", async () => {
    const catalog = session.catalog();
    try {
      expect(catalog.sessionId).toBe(session.sessionId);

      const schemas = await catalog.listSchemas();
      expect(schemas.map((s) => s.name)).toContain("default");

      const graphs = await catalog.listGraphs("default");
      expect(graphs.length).toBeGreaterThan(0);

      const info = await catalog.getGraphInfo("default", "default");
      expect(info.name).toBe("default");
    } finally {
      catalog.close();
    }
  });

  it("session-bound client shares the session's channel", () => {
    const catalog = session.catalog();
    // White-box: the channel is what a leak would leave open.
    const channelOf = (c: object) =>
      (c as { client: { getChannel(): unknown } }).client.getChannel();
    const sessionChannel = (
      session as unknown as { sessionClient: { getChannel(): unknown } }
    ).sessionClient.getChannel();
    expect(channelOf(catalog)).toBe(sessionChannel);
    catalog.close();
  });

  it("closing a session-bound client leaves the session usable", async () => {
    session.catalog().close();
    const schemas = await session.catalog().listSchemas();
    expect(schemas.map((s) => s.name)).toContain("default");
    expect(await session.ping()).toBeGreaterThan(0);
  });

  it("connection-level client names no session", async () => {
    const catalog = conn.createCatalogClient();
    try {
      expect(catalog.sessionId).toBeNull();

      const schemas = await catalog.listSchemas();
      expect(schemas.map((s) => s.name)).toContain("default");
    } finally {
      catalog.close();
    }
  });

  it("client bound to an unknown session gets NOT_FOUND", async () => {
    const catalog = new CatalogClient(
      endpoint,
      credentials.createInsecure(),
      "no-such-session",
    );
    try {
      expect(catalog.sessionId).toBe("no-such-session");
      const err = await catalog.listSchemas().then(
        () => null,
        (e: ServiceError) => e,
      );
      expect(err).not.toBeNull();
      expect(err!.code).toBe(status.NOT_FOUND);
    } finally {
      catalog.close();
    }
  });
});
