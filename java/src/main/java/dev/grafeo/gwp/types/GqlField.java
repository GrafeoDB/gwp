package dev.grafeo.gwp.types;

/**
 * A single named field within a {@link GqlRecord}.
 *
 * @param name  the field name
 * @param value the field value (may be any GQL-compatible Java type or null)
 */
public record GqlField(String name, Object value) {
}
