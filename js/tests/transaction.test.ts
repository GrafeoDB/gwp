import { describe, it, expect, beforeAll, afterAll } from "vitest";
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

describe("transaction", () => {
  it("begin and commit", async () => {
    const tx = await session.beginTransaction();
    const cursor = await tx.execute("INSERT INTO t VALUES (1)");
    await cursor.collectRows();
    await tx.commit();
  });

  it("begin and rollback", async () => {
    const tx = await session.beginTransaction();
    const cursor = await tx.execute("INSERT INTO t VALUES (1)");
    await cursor.collectRows();
    await tx.rollback();
  });

  it("execute MATCH within transaction", async () => {
    const tx = await session.beginTransaction();
    const cursor = await tx.execute("MATCH (n:Person) RETURN n.name");
    const cols = await cursor.columnNames();
    expect(cols).toContain("name");
    const rows = await cursor.collectRows();
    expect(rows).toHaveLength(2);
    await tx.commit();
  });

  it("rollback is no-op after commit", async () => {
    const tx = await session.beginTransaction();
    await tx.commit();
    await tx.rollback(); // should not throw
  });
});
