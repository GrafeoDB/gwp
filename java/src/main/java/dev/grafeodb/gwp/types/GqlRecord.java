package io.grafeodb.gwp.types;

import java.util.List;
import java.util.Optional;

/**
 * An ordered collection of named fields (open or closed record).
 *
 * @param fields the list of fields in order
 */
public record GqlRecord(List<GqlField> fields) {

    /**
     * Get a field value by name.
     *
     * @param name the field name to look up
     * @return an Optional containing the value if found, or empty
     */
    public Optional<Object> get(String name) {
        for (GqlField field : fields) {
            if (field.name().equals(name)) {
                return Optional.ofNullable(field.value());
            }
        }
        return Optional.empty();
    }

    /**
     * Get a field value by name, throwing if not found.
     *
     * @param name the field name to look up
     * @return the field value
     * @throws IllegalArgumentException if the field is not found
     */
    public Object require(String name) {
        for (GqlField field : fields) {
            if (field.name().equals(name)) {
                return field.value();
            }
        }
        throw new IllegalArgumentException("field not found: " + name);
    }

    /**
     * The number of fields in the record.
     *
     * @return the field count
     */
    public int size() {
        return fields.size();
    }
}
