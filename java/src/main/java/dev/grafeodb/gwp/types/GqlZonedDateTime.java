package io.grafeodb.gwp.types;

import java.time.OffsetDateTime;
import java.time.ZoneOffset;

/**
 * A GQL datetime value with UTC offset.
 *
 * @param date          the date component
 * @param time          the time component
 * @param offsetMinutes the UTC offset in minutes
 */
public record GqlZonedDateTime(GqlDate date, GqlLocalTime time, int offsetMinutes) {

    /**
     * Convert to a Java {@link OffsetDateTime}.
     *
     * @return the equivalent OffsetDateTime
     */
    public OffsetDateTime toOffsetDateTime() {
        ZoneOffset offset = ZoneOffset.ofTotalSeconds(offsetMinutes * 60);
        return OffsetDateTime.of(date.toLocalDate(), time.toLocalTime(), offset);
    }
}
