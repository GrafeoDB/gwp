package dev.grafeo.gwp.types;

import java.time.LocalDate;

/**
 * A GQL date value.
 *
 * @param year  the year
 * @param month the month (1-12)
 * @param day   the day (1-31)
 */
public record GqlDate(int year, int month, int day) {

    /**
     * Convert to a Java {@link LocalDate}.
     *
     * @return the equivalent LocalDate
     */
    public LocalDate toLocalDate() {
        return LocalDate.of(year, month, day);
    }
}
