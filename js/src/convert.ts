/** Convert between protobuf messages and native TypeScript types. */

import type { Value } from "./generated/gql_types";
import type {
  GqlDate,
  GqlDuration,
  GqlEdge,
  GqlField,
  GqlLocalDateTime,
  GqlLocalTime,
  GqlNode,
  GqlPath,
  GqlRecord,
  GqlValue,
  GqlZonedDateTime,
  GqlZonedTime,
} from "./types";

/** Convert a protobuf Value to a native GqlValue. */
export function valueFromProto(v: Value): GqlValue {
  if (v.nullValue !== undefined) return null;
  if (v.booleanValue !== undefined) return v.booleanValue;
  if (v.integerValue !== undefined) return v.integerValue;
  if (v.unsignedIntegerValue !== undefined) return v.unsignedIntegerValue;
  if (v.floatValue !== undefined) return v.floatValue;
  if (v.stringValue !== undefined) return v.stringValue;
  if (v.bytesValue !== undefined) return v.bytesValue;

  if (v.dateValue) {
    return {
      year: v.dateValue.year,
      month: v.dateValue.month,
      day: v.dateValue.day,
    } satisfies GqlDate;
  }

  if (v.localTimeValue) {
    return {
      hour: v.localTimeValue.hour,
      minute: v.localTimeValue.minute,
      second: v.localTimeValue.second,
      nanosecond: v.localTimeValue.nanosecond,
    } satisfies GqlLocalTime;
  }

  if (v.zonedTimeValue) {
    const t = v.zonedTimeValue.time!;
    return {
      time: {
        hour: t.hour,
        minute: t.minute,
        second: t.second,
        nanosecond: t.nanosecond,
      },
      offsetMinutes: v.zonedTimeValue.offsetMinutes,
    } satisfies GqlZonedTime;
  }

  if (v.localDatetimeValue) {
    const d = v.localDatetimeValue.date!;
    const t = v.localDatetimeValue.time!;
    return {
      date: { year: d.year, month: d.month, day: d.day },
      time: {
        hour: t.hour,
        minute: t.minute,
        second: t.second,
        nanosecond: t.nanosecond,
      },
    } satisfies GqlLocalDateTime;
  }

  if (v.zonedDatetimeValue) {
    const d = v.zonedDatetimeValue.date!;
    const t = v.zonedDatetimeValue.time!;
    return {
      date: { year: d.year, month: d.month, day: d.day },
      time: {
        hour: t.hour,
        minute: t.minute,
        second: t.second,
        nanosecond: t.nanosecond,
      },
      offsetMinutes: v.zonedDatetimeValue.offsetMinutes,
    } satisfies GqlZonedDateTime;
  }

  if (v.durationValue) {
    return {
      months: v.durationValue.months,
      nanoseconds: v.durationValue.nanoseconds,
    } satisfies GqlDuration;
  }

  if (v.listValue) {
    return v.listValue.elements.map(valueFromProto);
  }

  if (v.recordValue) {
    const fields: GqlField[] = v.recordValue.fields.map((f) => ({
      name: f.name,
      value: f.value ? valueFromProto(f.value) : null,
    }));
    return { fields } satisfies GqlRecord;
  }

  if (v.nodeValue) {
    const props = new Map<string, GqlValue>();
    for (const [k, pv] of Object.entries(v.nodeValue.properties)) {
      props.set(k, valueFromProto(pv));
    }
    return {
      id: v.nodeValue.id,
      labels: v.nodeValue.labels,
      properties: props,
    } satisfies GqlNode;
  }

  if (v.edgeValue) {
    const props = new Map<string, GqlValue>();
    for (const [k, pv] of Object.entries(v.edgeValue.properties)) {
      props.set(k, valueFromProto(pv));
    }
    return {
      id: v.edgeValue.id,
      labels: v.edgeValue.labels,
      sourceNodeId: v.edgeValue.sourceNodeId,
      targetNodeId: v.edgeValue.targetNodeId,
      undirected: v.edgeValue.undirected,
      properties: props,
    } satisfies GqlEdge;
  }

  if (v.pathValue) {
    const nodes: GqlNode[] = v.pathValue.nodes.map((n) => {
      const props = new Map<string, GqlValue>();
      for (const [k, pv] of Object.entries(n.properties)) {
        props.set(k, valueFromProto(pv));
      }
      return { id: n.id, labels: n.labels, properties: props };
    });
    const edges: GqlEdge[] = v.pathValue.edges.map((e) => {
      const props = new Map<string, GqlValue>();
      for (const [k, pv] of Object.entries(e.properties)) {
        props.set(k, valueFromProto(pv));
      }
      return {
        id: e.id,
        labels: e.labels,
        sourceNodeId: e.sourceNodeId,
        targetNodeId: e.targetNodeId,
        undirected: e.undirected,
        properties: props,
      };
    });
    return { nodes, edges } satisfies GqlPath;
  }

  // BigInteger, BigFloat, Decimal - not supported in v0.1
  return null;
}

/** Convert a native value to a protobuf Value. */
export function valueToProto(value: GqlValue): Value {
  if (value === null) return { nullValue: {} };
  if (typeof value === "boolean") return { booleanValue: value };
  if (typeof value === "bigint") return { integerValue: value };
  if (typeof value === "number") return { floatValue: value };
  if (typeof value === "string") return { stringValue: value };
  if (value instanceof Uint8Array) return { bytesValue: value };
  if (Array.isArray(value)) {
    return { listValue: { elements: value.map(valueToProto) } };
  }
  return { nullValue: {} };
}
