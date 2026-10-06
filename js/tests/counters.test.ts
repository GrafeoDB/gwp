import { describe, it, expect, beforeAll, afterAll } from "vitest";
import { GqlConnection } from "../src/connection";
import { GqlSession } from "../src/session";
import { Counters, COUNTER_KEYS } from "../src/counters";
import { getTestServer, stopTestServer } from "./helpers";

describe("Counters", () => {
  it("missing keys read as zero", () => {
    const counters = Counters.fromMap({});
    expect(counters.nodesCreated).toBe(0n);
    expect(counters.labelsRemoved).toBe(0n);
    expect(counters.containsUpdates()).toBe(false);
  });

  it("reads every key and ignores others", () => {
    const map = new Map<string, bigint>([
      ["execution_time_ms", 99n],
      ["rows_scanned", 1000n],
    ]);
    COUNTER_KEYS.forEach((key, i) => map.set(key, BigInt(i + 1)));

    const counters = Counters.fromMap(map);
    expect(counters.nodesCreated).toBe(1n);
    expect(counters.nodesDeleted).toBe(2n);
    expect(counters.edgesCreated).toBe(3n);
    expect(counters.edgesDeleted).toBe(4n);
    expect(counters.propertiesSet).toBe(5n);
    expect(counters.labelsAdded).toBe(6n);
    expect(counters.labelsRemoved).toBe(7n);
    expect(counters.containsUpdates()).toBe(true);
  });

  it("negative values read as zero", () => {
    expect(Counters.fromMap({ nodes_created: -5n }).nodesCreated).toBe(0n);
  });

  it("a single counter is an update", () => {
    for (const key of COUNTER_KEYS) {
      expect(Counters.fromMap({ [key]: 1n }).containsUpdates()).toBe(true);
    }
  });
});

describe("write counters from the server", () => {
  let conn: GqlConnection;
  let session: GqlSession;

  beforeAll(async () => {
    conn = GqlConnection.connect(await getTestServer());
    session = await conn.createSession();
  });

  afterAll(async () => {
    await session.close();
    conn.close();
    stopTestServer();
  });

  it("INSERT reports typed write counters", async () => {
    const cursor = await session.execute("INSERT (:Person {name: 'Alix'})");
    const counters = await cursor.counters();
    expect(counters.nodesCreated).toBe(3n);
    expect(counters.labelsAdded).toBe(3n);
    expect(counters.propertiesSet).toBe(6n);
    expect(counters.nodesDeleted).toBe(0n);
    expect(counters.containsUpdates()).toBe(true);

    // Other entries stay in the raw map.
    const summary = await cursor.summary();
    expect(summary?.counters.get("execution_time_ms")).toBe(1n);
    expect(summary?.writeCounters).toEqual(counters);
  });

  it("a read writes nothing", async () => {
    const cursor = await session.execute("MATCH (n) RETURN n");
    await cursor.collectRows();
    expect((await cursor.counters()).containsUpdates()).toBe(false);
  });
});
