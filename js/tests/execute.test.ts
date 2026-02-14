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

describe("execute", () => {
  it("MATCH returns columns and rows", async () => {
    const cursor = await session.execute("MATCH (n:Person) RETURN n.name, n.age");
    const cols = await cursor.columnNames();
    expect(cols).toEqual(["name", "age"]);

    const rows = await cursor.collectRows();
    expect(rows).toHaveLength(2);
    expect(rows[0][0]).toBe("Alice");
    expect(rows[0][1]).toBe(30n);
    expect(rows[1][0]).toBe("Bob");
    expect(rows[1][1]).toBe(25n);
  });

  it("async iterator", async () => {
    const cursor = await session.execute("MATCH (n) RETURN n");
    const names: unknown[] = [];
    for await (const row of cursor) {
      names.push(row[0]);
    }
    expect(names).toEqual(["Alice", "Bob"]);
  });

  it("DDL returns empty rows", async () => {
    const cursor = await session.execute("CREATE GRAPH mygraph");
    const rows = await cursor.collectRows();
    expect(rows).toEqual([]);
    const summary = await cursor.summary();
    expect(summary).not.toBeNull();
  });

  it("DML returns rows_affected", async () => {
    const cursor = await session.execute("INSERT INTO t VALUES (1)");
    const rows = await cursor.collectRows();
    expect(rows).toEqual([]);
    const affected = await cursor.rowsAffected();
    expect(affected).toBe(3n);
  });

  it("is_success on MATCH", async () => {
    const cursor = await session.execute("MATCH (n) RETURN n");
    expect(await cursor.isSuccess()).toBe(true);
  });
});
