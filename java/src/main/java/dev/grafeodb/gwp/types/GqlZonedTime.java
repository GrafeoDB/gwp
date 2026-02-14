package dev.grafeodb.gwp.types;

import java.time.LocalTime;
import java.time.OffsetTime;
import java.time.ZoneOffset;

/**
 * A GQL time value with UTC offset.
 *
 * @param time          the local time component
 * @param offsetMinutes the UTC offset in minutes
 */
public record GqlZonedTime(GqlLocalTime time, int offsetMinutes) {

    /**
     * Convert to a Java {@link OffsetTime}.
     *
     * @return the equivalent OffsetTime
     */
    public OffsetTime toOffsetTime() {
        LocalTime lt = time.toLocalTime();
        ZoneOffset offset = ZoneOffset.ofTotalSeconds(offsetMinutes * 60);
        return OffsetTime.of(lt, offset);
    }
}
