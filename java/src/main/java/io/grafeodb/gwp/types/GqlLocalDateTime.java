package io.grafeodb.gwp.types;

import java.time.LocalDateTime;

/**
 * A GQL local datetime value (no timezone).
 *
 * @param date the date component
 * @param time the time component
 */
public record GqlLocalDateTime(GqlDate date, GqlLocalTime time) {

    /**
     * Convert to a Java {@link LocalDateTime}.
     *
     * @return the equivalent LocalDateTime
     */
    public LocalDateTime toLocalDateTime() {
        return LocalDateTime.of(date.toLocalDate(), time.toLocalTime());
    }
}
