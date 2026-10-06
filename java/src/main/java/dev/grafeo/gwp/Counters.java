package dev.grafeo.gwp;

import java.util.List;
import java.util.Map;

/**
 * The write counters of a statement, read from {@code ResultSummary.counters}.
 *
 * <p>The summary's counter map uses the keys in {@link #KEYS}; a missing key
 * counts as 0. Other entries of that map (such as {@code execution_time_ms})
 * are not write counters and are left out; they stay available through
 * {@link ResultCursor.ResultSummary#counters()}.</p>
 *
 * @param nodesCreated nodes created
 * @param nodesDeleted nodes deleted
 * @param edgesCreated edges created
 * @param edgesDeleted edges deleted
 * @param propertiesSet property values set or removed
 * @param labelsAdded labels added
 * @param labelsRemoved labels removed
 */
public record Counters(
        long nodesCreated,
        long nodesDeleted,
        long edgesCreated,
        long edgesDeleted,
        long propertiesSet,
        long labelsAdded,
        long labelsRemoved) {

    /** Key of {@link #nodesCreated()}. */
    public static final String NODES_CREATED = "nodes_created";
    /** Key of {@link #nodesDeleted()}. */
    public static final String NODES_DELETED = "nodes_deleted";
    /** Key of {@link #edgesCreated()}. */
    public static final String EDGES_CREATED = "edges_created";
    /** Key of {@link #edgesDeleted()}. */
    public static final String EDGES_DELETED = "edges_deleted";
    /** Key of {@link #propertiesSet()}. */
    public static final String PROPERTIES_SET = "properties_set";
    /** Key of {@link #labelsAdded()}. */
    public static final String LABELS_ADDED = "labels_added";
    /** Key of {@link #labelsRemoved()}. */
    public static final String LABELS_REMOVED = "labels_removed";

    /** All write counter keys, in field order. */
    public static final List<String> KEYS = List.of(
            NODES_CREATED,
            NODES_DELETED,
            EDGES_CREATED,
            EDGES_DELETED,
            PROPERTIES_SET,
            LABELS_ADDED,
            LABELS_REMOVED);

    /** Counters that are all zero. */
    public static final Counters NONE = new Counters(0, 0, 0, 0, 0, 0, 0);

    /**
     * Read the write counters from a summary's counter map.
     *
     * <p>Missing keys read as 0, and so does a negative value.</p>
     *
     * @param counters the summary's counter map
     * @return the typed write counters
     */
    public static Counters fromMap(Map<String, Long> counters) {
        return new Counters(
                get(counters, NODES_CREATED),
                get(counters, NODES_DELETED),
                get(counters, EDGES_CREATED),
                get(counters, EDGES_DELETED),
                get(counters, PROPERTIES_SET),
                get(counters, LABELS_ADDED),
                get(counters, LABELS_REMOVED));
    }

    /**
     * Whether any counter is non-zero, that is, whether the statement changed the graph.
     *
     * @return true if the statement wrote something
     */
    public boolean containsUpdates() {
        return nodesCreated > 0 || nodesDeleted > 0 || edgesCreated > 0 || edgesDeleted > 0
                || propertiesSet > 0 || labelsAdded > 0 || labelsRemoved > 0;
    }

    private static long get(Map<String, Long> counters, String key) {
        Long value = counters.get(key);
        return value != null && value > 0 ? value : 0;
    }
}
