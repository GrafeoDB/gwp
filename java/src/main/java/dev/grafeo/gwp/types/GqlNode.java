package dev.grafeodb.gwp.types;

import java.util.List;
import java.util.Map;

/**
 * A property graph node.
 *
 * @param id         the opaque element identifier
 * @param labels     the label set (unordered)
 * @param properties the property map
 */
public record GqlNode(byte[] id, List<String> labels, Map<String, Object> properties) {

    /**
     * Check if this node has the given label.
     *
     * @param label the label to check
     * @return true if the node has the label
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
