package dev.grafeodb.gwp;

import dev.grafeodb.gwp.internal.ValueConverter;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.Map;
import java.util.NoSuchElementException;

/**
 * Cursor over streaming result frames from an Execute RPC.
 *
 * <p>Implements {@link Iterator} over rows (each row is a {@code List<Object>})
 * and {@link AutoCloseable} for use with try-with-resources.</p>
 *
 * <p>This is a blocking cursor. Rows are consumed from the gRPC response stream
 * on demand. The cursor buffers rows from each {@code RowBatch} frame.</p>
 */
public class ResultCursor implements Iterator<List<Object>>, AutoCloseable {

    private final Iterator<gql.GqlService.ExecuteResponse> stream;
    private gql.GqlService.ResultHeader header;
    private gql.GqlService.ResultSummary protoSummary;
    private final List<List<Object>> bufferedRows = new ArrayList<>();
    private int rowIndex = 0;
    private boolean done = false;

    /**
     * Create a new ResultCursor wrapping a gRPC response stream.
     *
     * @param stream the response stream iterator from an Execute RPC
     */
    public ResultCursor(Iterator<gql.GqlService.ExecuteResponse> stream) {
        this.stream = stream;
    }

    // ========================================================================
    // Column metadata
    // ========================================================================

    /**
     * Get the column names from the result header.
     *
     * <p>Consumes frames from the stream if the header has not been received yet.</p>
     *
     * @return the list of column names, or an empty list if no header
     */
    public List<String> columnNames() {
        if (header == null) {
            consumeUntilRowsOrDone();
        }
        if (header == null) {
            return List.of();
        }
        List<String> names = new ArrayList<>(header.getColumnsCount());
        for (gql.GqlService.ColumnDescriptor col : header.getColumnsList()) {
            names.add(col.getName());
        }
        return names;
    }

    // ========================================================================
    // Row access
    // ========================================================================

    /**
     * Get the next row. Returns {@code null} when there are no more rows.
     *
     * @return the next row as a list of native Java objects, or null if done
     */
    public List<Object> nextRow() {
        if (rowIndex < bufferedRows.size()) {
            return bufferedRows.get(rowIndex++);
        }

        consumeUntilRowsOrDone();

        if (rowIndex < bufferedRows.size()) {
            return bufferedRows.get(rowIndex++);
        }

        return null;
    }

    /**
     * Collect all remaining rows into a list.
     *
     * @return all remaining rows
     */
    public List<List<Object>> collectRows() {
        List<List<Object>> rows = new ArrayList<>();
        List<Object> row;
        while ((row = nextRow()) != null) {
            rows.add(row);
        }
        return rows;
    }

    // ========================================================================
    // Summary
    // ========================================================================

    /**
     * Get the result summary. Consumes all remaining frames if needed.
     *
     * @return the result summary, or null if the stream ended without one
     */
    public ResultSummary summary() {
        while (!done) {
            rowIndex = bufferedRows.size();
            consumeUntilRowsOrDone();
        }
        if (protoSummary != null) {
            return new ResultSummary(protoSummary);
        }
        return null;
    }

    /**
     * Check if the execution was successful.
     *
     * @return true if the result summary indicates success
     */
    public boolean isSuccess() {
        ResultSummary s = summary();
        return s != null && s.isSuccess();
    }

    /**
     * Get the number of rows affected.
     *
     * @return the row count, or 0 if no summary
     */
    public long rowsAffected() {
        ResultSummary s = summary();
        return s != null ? s.rowsAffected() : 0;
    }

    // ========================================================================
    // Iterator implementation
    // ========================================================================

    @Override
    public boolean hasNext() {
        if (rowIndex < bufferedRows.size()) {
            return true;
        }
        if (done) {
            return false;
        }
        consumeUntilRowsOrDone();
        return rowIndex < bufferedRows.size();
    }

    @Override
    public List<Object> next() {
        List<Object> row = nextRow();
        if (row == null) {
            throw new NoSuchElementException("no more rows");
        }
        return row;
    }

    // ========================================================================
    // AutoCloseable
    // ========================================================================

    @Override
    public void close() {
        // Drain remaining frames to release gRPC resources
        while (!done) {
            rowIndex = bufferedRows.size();
            consumeUntilRowsOrDone();
        }
    }

    // ========================================================================
    // Internal
    // ========================================================================

    private void consumeUntilRowsOrDone() {
        while (!done && rowIndex >= bufferedRows.size()) {
            if (!stream.hasNext()) {
                done = true;
                return;
            }

            gql.GqlService.ExecuteResponse response = stream.next();
            gql.GqlService.ExecuteResponse.FrameCase frame = response.getFrameCase();

            switch (frame) {
                case HEADER -> header = response.getHeader();

                case ROW_BATCH -> {
                    for (gql.GqlService.Row row : response.getRowBatch().getRowsList()) {
                        List<Object> nativeRow = new ArrayList<>(row.getValuesCount());
                        for (gql.GqlTypes.Value v : row.getValuesList()) {
                            nativeRow.add(ValueConverter.fromProto(v));
                        }
                        bufferedRows.add(nativeRow);
                    }
                }

                case SUMMARY -> {
                    protoSummary = response.getSummary();
                    done = true;
                }

                default -> {
                    // Unknown frame type, skip
                }
            }
        }
    }

    // ========================================================================
    // Result Summary (nested class)
    // ========================================================================

    /**
     * Summary of a completed query execution.
     */
    public static class ResultSummary {

        private final gql.GqlService.ResultSummary proto;

        ResultSummary(gql.GqlService.ResultSummary proto) {
            this.proto = proto;
        }

        /** The 5-character GQLSTATUS code. */
        public String statusCode() {
            return proto.hasStatus() ? proto.getStatus().getCode() : "";
        }

        /** The human-readable status message. */
        public String message() {
            return proto.hasStatus() ? proto.getStatus().getMessage() : "";
        }

        /** Number of rows affected. */
        public long rowsAffected() {
            return proto.getRowsAffected();
        }

        /** Operation counters (e.g. nodes_created, edges_deleted). */
        public Map<String, Long> counters() {
            return proto.getCountersMap();
        }

        /** Check if the execution was successful. */
        public boolean isSuccess() {
            return GqlStatus.isSuccess(statusCode());
        }
    }
}
