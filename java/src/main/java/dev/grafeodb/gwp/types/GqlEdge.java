package io.grafeodb.gwp.types;

import java.util.List;
import java.util.Map;

/**
 * A property graph edge (directed or undirected).
 *
 * @param id           the opaque element identifier
 * @param labels       the label set
 * @param sourceNodeId the source node ID (directed) or endpoint A
 * @param targetNodeId the target node ID (directed) or endpoint B
 * @param undirected   true if this is an undirected edge
 * @param properties   the property map
 */
public record GqlEdge(
        byte[] id,
        List<String> labels,
        byte[] sourceNodeId,
        byte[] targetNodeId,
        boolean undirected,
        Map<String, Object> properties) {

    /**
     * Check if this edge has the given label.
     *
     * @param label the label to check
     * @return true if the edge has the label
     */
    public boolean hasLabel(String label) {
        return labels.contains(label);
    }

    /**
     * Get a property value by key.
     *
     * @param key the property key
     * @return the property value, or null if not found
     */
    public Object property(String key) {
        return properties.get(key);
    }
}
