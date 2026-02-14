package dev.grafeodb.gwp.types;

/**
 * A GQL duration value with two components per ISO/IEC 39075.
 *
 * @param months      the year-to-month component
 * @param nanoseconds the day-to-second component in nanoseconds
 */
public record GqlDuration(long months, long nanoseconds) {
}
