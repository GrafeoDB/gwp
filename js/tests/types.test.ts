import { describe, it, expect } from "vitest";
import { valueFromProto, valueToProto } from "../src/convert";
import type { GqlNode, GqlEdge, GqlPath, GqlRecord } from "../src/types";

describe("value conversion", () => {
  it("null roundtrip", () => {
    const proto = valueToProto(null);
    expect(valueFromProto(proto)).toBeNull();
  });

  it("boolean roundtrip", () => {
    const proto = valueToProto(true);
    expect(valueFromProto(proto)).toBe(true);
  });

  it("integer roundtrip", () => {
    const proto = valueToProto(42n);
    expect(valueFromProto(proto)).toBe(42n);
  });

  it("float roundtrip", () => {
    const proto = valueToProto(3.14);
    expect(valueFromProto(proto)).toBeCloseTo(3.14);
  });

  it("string roundtrip", () => {
    const proto = valueToProto("hello");
    expect(valueFromProto(proto)).toBe("hello");
  });

  it("bytes roundtrip", () => {
    const data = new Uint8Array([1, 2, 3]);
    const proto = valueToProto(data);
    expect(valueFromProto(proto)).toEqual(data);
  });

  it("list roundtrip", () => {
    const proto = valueToProto([1n, "two", null]);
    const result = valueFromProto(proto) as any[];
    expect(result).toHaveLength(3);
    expect(result[0]).toBe(1n);
    expect(result[1]).toBe("two");
    expect(result[2]).toBeNull();
  });
});

describe("proto to native - graph elements", () => {
  it("node from proto", () => {
    const proto = {
      nodeValue: {
        id: new Uint8Array([1]),
        labels: ["Person"],
        properties: {
          name: { stringValue: "Alice" },
        },
      },
    };
    const node = valueFromProto(proto as any) as GqlNode;
    expect(node.labels).toEqual(["Person"]);
    expect(node.properties.get("name")).toBe("Alice");
  });

  it("edge from proto", () => {
    const proto = {
      edgeValue: {
        id: new Uint8Array([16]),
        labels: ["knows"],
        sourceNodeId: new Uint8Array([1]),
        targetNodeId: new Uint8Array([2]),
        undirected: false,
        properties: {},
      },
    };
    const edge = valueFromProto(proto as any) as GqlEdge;
    expect(edge.labels).toEqual(["knows"]);
    expect(edge.undirected).toBe(false);
  });
});

describe("proto to native - temporal types", () => {
  it("date from proto", () => {
    const proto = {
      dateValue: { year: 2024, month: 3, day: 15 },
    };
    const d = valueFromProto(proto as any) as { year: number; month: number; day: number };
    expect(d.year).toBe(2024);
    expect(d.month).toBe(3);
    expect(d.day).toBe(15);
  });

  it("local time from proto", () => {
    const proto = {
      localTimeValue: { hour: 14, minute: 30, second: 0, nanosecond: 0 },
    };
    const t = valueFromProto(proto as any) as { hour: number; minute: number };
    expect(t.hour).toBe(14);
    expect(t.minute).toBe(30);
  });

  it("duration from proto", () => {
    const proto = {
      durationValue: { months: 12n, nanoseconds: 500_000_000n },
    };
    const d = valueFromProto(proto as any) as { months: bigint; nanoseconds: bigint };
    expect(d.months).toBe(12n);
    expect(d.nanoseconds).toBe(500_000_000n);
  });
});
