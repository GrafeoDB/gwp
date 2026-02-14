/** gwp-js - TypeScript client for the GQL Wire Protocol (GWP). */

export { GqlConnection } from "./connection";
export { GqlSession } from "./session";
export { ResultCursor, ResultSummary } from "./result";
export { Transaction } from "./transaction";

export {
  GqlError,
  ConnectionError,
  SessionError,
  TransactionError,
  GqlStatusError,
} from "./errors";

export {
  SUCCESS,
  OMITTED_RESULT,
  WARNING,
  NO_DATA,
  INVALID_SYNTAX,
  GRAPH_TYPE_VIOLATION,
  statusClass,
  isSuccess,
  isWarning,
  isNoData,
  isException,
} from "./status";

export type {
  GqlNode,
  GqlEdge,
  GqlPath,
  GqlField,
  GqlRecord,
  GqlDate,
  GqlLocalTime,
  GqlZonedTime,
  GqlLocalDateTime,
  GqlZonedDateTime,
  GqlDuration,
  GqlValue,
} from "./types";
