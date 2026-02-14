/** Streaming result cursor. */

import type { ClientReadableStream } from "@grpc/grpc-js";
import type { ExecuteResponse } from "./generated/gql_service";
import { valueFromProto } from "./convert";
import { isSuccess } from "./status";
import type { GqlValue } from "./types";

/** Summary of a completed query execution. */
export class ResultSummary {
  /** @internal */
  constructor(private readonly proto: NonNullable<ExecuteResponse["summary"]>) {}

  /** The GQLSTATUS code. */
  get statusCode(): string {
    return this.proto.status?.code ?? "";
  }

  /** The status message. */
  get message(): string {
    return this.proto.status?.message ?? "";
  }

  /** Number of rows affected. */
  get rowsAffected(): bigint {
    return this.proto.rowsAffected;
  }

  /** Operation counters. */
  get counters(): ReadonlyMap<string, bigint> {
    return new Map(Object.entries(this.proto.counters));
  }

  /** Check if the execution was successful. */
  isSuccess(): boolean {
    return isSuccess(this.statusCode);
  }
}

/** Cursor over streaming result frames from an Execute RPC. */
export class ResultCursor implements AsyncIterable<GqlValue[]> {
  private readonly stream: ClientReadableStream<ExecuteResponse>;
  private header: ExecuteResponse["header"] | undefined;
  private _summary: ExecuteResponse["summary"] | undefined;
  private bufferedRows: GqlValue[][] = [];
  private rowIndex = 0;
  private done = false;
  private pendingResolve: (() => void) | null = null;

  /** @internal */
  constructor(stream: ClientReadableStream<ExecuteResponse>) {
    this.stream = stream;

    stream.on("data", (response: ExecuteResponse) => {
      if (response.header) {
        this.header = response.header;
      } else if (response.rowBatch) {
        for (const row of response.rowBatch.rows) {
          this.bufferedRows.push(row.values.map(valueFromProto));
        }
      } else if (response.summary) {
        this._summary = response.summary;
        this.done = true;
      }

      if (this.pendingResolve) {
        const resolve = this.pendingResolve;
        this.pendingResolve = null;
        resolve();
      }
    });

    stream.on("end", () => {
      this.done = true;
      if (this.pendingResolve) {
        const resolve = this.pendingResolve;
        this.pendingResolve = null;
        resolve();
      }
    });

    stream.on("error", () => {
      this.done = true;
      if (this.pendingResolve) {
        const resolve = this.pendingResolve;
        this.pendingResolve = null;
        resolve();
      }
    });
  }

  private waitForData(): Promise<void> {
    if (this.done || this.rowIndex < this.bufferedRows.length) {
      return Promise.resolve();
    }
    return new Promise<void>((resolve) => {
      this.pendingResolve = () => resolve();
    });
  }

  private waitForDone(): Promise<void> {
    if (this.done) return Promise.resolve();
    return new Promise<void>((resolve) => {
      this.pendingResolve = () => {
        if (this.done) resolve();
        else this.waitForDone().then(resolve);
      };
    });
  }

  /** Get the column names from the result header. */
  async columnNames(): Promise<string[]> {
    while (!this.header && !this.done) {
      await this.waitForData();
    }
    return this.header?.columns.map((c) => c.name) ?? [];
  }

  /** Get the next row. Returns null when done. */
  async nextRow(): Promise<GqlValue[] | null> {
    if (this.rowIndex < this.bufferedRows.length) {
      return this.bufferedRows[this.rowIndex++];
    }

    while (!this.done) {
      await this.waitForData();
      if (this.rowIndex < this.bufferedRows.length) {
        return this.bufferedRows[this.rowIndex++];
      }
    }

    return null;
  }

  /** Collect all remaining rows. */
  async collectRows(): Promise<GqlValue[][]> {
    const rows: GqlValue[][] = [];
    let row = await this.nextRow();
    while (row !== null) {
      rows.push(row);
      row = await this.nextRow();
    }
    return rows;
  }

  /** Get the result summary. Consumes remaining frames if needed. */
  async summary(): Promise<ResultSummary | null> {
    await this.waitForDone();
    return this._summary ? new ResultSummary(this._summary) : null;
  }

  /** Check if the execution was successful. */
  async isSuccess(): Promise<boolean> {
    const s = await this.summary();
    return s?.isSuccess() ?? false;
  }

  /** Get the number of rows affected. */
  async rowsAffected(): Promise<bigint> {
    const s = await this.summary();
    return s?.rowsAffected ?? 0n;
  }

  async *[Symbol.asyncIterator](): AsyncIterator<GqlValue[]> {
    let row = await this.nextRow();
    while (row !== null) {
      yield row;
      row = await this.nextRow();
    }
  }
}
