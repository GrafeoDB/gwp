/** Typed write counters carried in `ResultSummary.counters`. */

/** Write counter keys in the summary's counter map. A missing key counts as 0. */
export const COUNTER_KEYS = [
  "nodes_created",
  "nodes_deleted",
  "edges_created",
  "edges_deleted",
  "properties_set",
  "labels_added",
  "labels_removed",
] as const;

/** One of the write counter keys. */
export type CounterKey = (typeof COUNTER_KEYS)[number];

/**
 * The write counters of a statement, read from the summary's counter map.
 *
 * Other entries of that map (such as `execution_time_ms`) are not write
 * counters and are left out; they stay available on `ResultSummary.counters`.
 */
export class Counters {
  readonly nodesCreated: bigint;
  readonly nodesDeleted: bigint;
  readonly edgesCreated: bigint;
  readonly edgesDeleted: bigint;
  readonly propertiesSet: bigint;
  readonly labelsAdded: bigint;
  readonly labelsRemoved: bigint;

  constructor(values: Partial<Record<CounterKey, bigint>> = {}) {
    const get = (key: CounterKey): bigint => {
      const value = values[key] ?? 0n;
      return value > 0n ? value : 0n;
    };
    this.nodesCreated = get("nodes_created");
    this.nodesDeleted = get("nodes_deleted");
    this.edgesCreated = get("edges_created");
    this.edgesDeleted = get("edges_deleted");
    this.propertiesSet = get("properties_set");
    this.labelsAdded = get("labels_added");
    this.labelsRemoved = get("labels_removed");
  }

  /**
   * Read the write counters from a summary's counter map. Missing keys read
   * as 0, and so does a negative value.
   */
  static fromMap(
    map: ReadonlyMap<string, bigint> | Readonly<Record<string, bigint>>,
  ): Counters {
    const entries: Iterable<[string, bigint]> =
      map instanceof Map ? map.entries() : Object.entries(map);
    const values: Partial<Record<CounterKey, bigint>> = {};
    for (const [key, value] of entries) {
      if ((COUNTER_KEYS as readonly string[]).includes(key)) {
        values[key as CounterKey] = value;
      }
    }
    return new Counters(values);
  }

  /** Whether any counter is non-zero, that is, whether the statement changed the graph. */
  containsUpdates(): boolean {
    return (
      this.nodesCreated > 0n ||
      this.nodesDeleted > 0n ||
      this.edgesCreated > 0n ||
      this.edgesDeleted > 0n ||
      this.propertiesSet > 0n ||
      this.labelsAdded > 0n ||
      this.labelsRemoved > 0n
    );
  }
}
