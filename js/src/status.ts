/** GQLSTATUS constants and helpers (ISO/IEC 39075 Chapter 23). */

// Success class (00)
export const SUCCESS = "00000";
export const OMITTED_RESULT = "00001";

// Warning class (01)
export const WARNING = "01000";

// No data class (02)
export const NO_DATA = "02000";

// Exception classes
export const INVALID_SYNTAX = "42001";
export const GRAPH_TYPE_VIOLATION = "G2000";

/** Extract the 2-character class from a 5-character GQLSTATUS code. */
export function statusClass(code: string): string {
  return code.slice(0, 2);
}

/** Check if the status indicates success (class 00). */
export function isSuccess(code: string): boolean {
  return statusClass(code) === "00";
}

/** Check if the status indicates a warning (class 01). */
export function isWarning(code: string): boolean {
  return statusClass(code) === "01";
}

/** Check if the status indicates no data (class 02). */
export function isNoData(code: string): boolean {
  return statusClass(code) === "02";
}

/** Check if the status indicates an exception (any class not 00, 01, 02). */
export function isException(code: string): boolean {
  const cls = statusClass(code);
  return cls !== "00" && cls !== "01" && cls !== "02";
}
