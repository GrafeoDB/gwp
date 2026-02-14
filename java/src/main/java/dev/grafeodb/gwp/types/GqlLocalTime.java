package io.grafeodb.gwp.types;

import java.time.LocalTime;

/**
 * A GQL local time value (no timezone).
 *
 * @param hour       the hour (0-23)
 * @param minute     the minute (0-59)
 * @param second     the second (0-59)
 * @param nanosecond the nanosecond (0-999999999)
 */
public record GqlLocalTime(int hour, int minute, int second, int nanosecond) {

    /**
     * Convert to a Java {@link LocalTime}.
     *
     * @return the equivalent LocalTime
     */
    public LocalTime toLocalTime() {
        return LocalTime.of(hour, minute, second, nanosecond);
    }
}
