/** GQL type wrappers for the JavaScript client. */

/** A property graph node. */
export interface GqlNode {
  readonly id: Uint8Array;
  readonly labels: readonly string[];
  readonly properties: ReadonlyMap<string, GqlValue>;
}

/** A property graph edge. */
export interface GqlEdge {
  readonly id: Uint8Array;
  readonly labels: readonly string[];
  readonly sourceNodeId: Uint8Array;
  readonly targetNodeId: Uint8Array;
  readonly undirected: boolean;
  readonly properties: ReadonlyMap<string, GqlValue>;
}

/** A path (alternating nodes and edges). */
export interface GqlPath {
  readonly nodes: readonly GqlNode[];
  readonly edges: readonly GqlEdge[];
}

/** A single field in a record. */
export interface GqlField {
  readonly name: string;
  readonly value: GqlValue;
}

/** A named collection of fields. */
export interface GqlRecord {
  readonly fields: readonly GqlField[];
}

/** Calendar date. */
export interface GqlDate {
  readonly year: number;
  readonly month: number;
  readonly day: number;
}

/** Time without timezone. */
export interface GqlLocalTime {
  readonly hour: number;
  readonly minute: number;
  readonly second: number;
  readonly nanosecond: number;
}

/** Time with UTC offset. */
export interface GqlZonedTime {
  readonly time: GqlLocalTime;
  readonly offsetMinutes: number;
}

/** Date and time without timezone. */
export interface GqlLocalDateTime {
  readonly date: GqlDate;
  readonly time: GqlLocalTime;
}

/** Date and time with UTC offset. */
export interface GqlZonedDateTime {
  readonly date: GqlDate;
  readonly time: GqlLocalTime;
  readonly offsetMinutes: number;
}

/** Temporal duration. */
export interface GqlDuration {
  readonly months: bigint;
  readonly nanoseconds: bigint;
}

/** Union of all native GQL values. */
export type GqlValue =
  | null
  | boolean
  | bigint
  | number
  | string
  | Uint8Array
  | GqlDate
  | GqlLocalTime
  | GqlZonedTime
  | GqlLocalDateTime
  | GqlZonedDateTime
  | GqlDuration
  | GqlValue[]
  | GqlRecord
  | GqlNode
  | GqlEdge
  | GqlPath;
