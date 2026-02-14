import { describe, it, expect } from "vitest";
import {
  SUCCESS,
  OMITTED_RESULT,
  WARNING,
  NO_DATA,
  INVALID_SYNTAX,
  GRAPH_TYPE_VIOLATION,
  isSuccess,
  isWarning,
  isNoData,
  isException,
  statusClass,
} from "../src/status";

describe("GQLSTATUS helpers", () => {
  it("success", () => {
    expect(isSuccess(SUCCESS)).toBe(true);
    expect(isException(SUCCESS)).toBe(false);
  });

  it("omitted", () => {
    expect(isSuccess(OMITTED_RESULT)).toBe(true);
  });

  it("warning", () => {
    expect(isWarning(WARNING)).toBe(true);
    expect(isSuccess(WARNING)).toBe(false);
    expect(isException(WARNING)).toBe(false);
  });

  it("no data", () => {
    expect(isNoData(NO_DATA)).toBe(true);
    expect(isSuccess(NO_DATA)).toBe(false);
  });

  it("exception", () => {
    expect(isException(INVALID_SYNTAX)).toBe(true);
    expect(isSuccess(INVALID_SYNTAX)).toBe(false);
  });

  it("graph type violation", () => {
    expect(isException(GRAPH_TYPE_VIOLATION)).toBe(true);
  });

  it("class extraction", () => {
    expect(statusClass("00000")).toBe("00");
    expect(statusClass("42001")).toBe("42");
    expect(statusClass("G2000")).toBe("G2");
  });
});
