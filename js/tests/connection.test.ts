import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { GqlConnection } from "../src/connection";
import { getTestServer, stopTestServer } from "./helpers";

let endpoint: string;

beforeAll(async () => {
  endpoint = await getTestServer();
});

afterAll(() => {
  stopTestServer();
});

describe("connection", () => {
  it("connect and create session", async () => {
    const conn = GqlConnection.connect(endpoint);
    const session = await conn.createSession();
    expect(session.sessionId).toMatch(/^mock-session-/);
    await session.close();
    conn.close();
  });

  it("ping", async () => {
    const conn = GqlConnection.connect(endpoint);
    const session = await conn.createSession();
    const ts = await session.ping();
    expect(ts).toBeGreaterThan(0n);
    await session.close();
    conn.close();
  });
});
