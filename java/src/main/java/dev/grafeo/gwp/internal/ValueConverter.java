package dev.grafeo.gwp.internal;

import com.google.protobuf.ByteString;

import dev.grafeo.gwp.types.GqlDate;
import dev.grafeo.gwp.types.GqlDuration;
import dev.grafeo.gwp.types.GqlEdge;
import dev.grafeo.gwp.types.GqlField;
import dev.grafeo.gwp.types.GqlLocalDateTime;
import dev.grafeo.gwp.types.GqlLocalTime;
import dev.grafeo.gwp.types.GqlNode;
import dev.grafeo.gwp.types.GqlPath;
import dev.grafeo.gwp.types.GqlRecord;
import dev.grafeo.gwp.types.GqlZonedDateTime;
import dev.grafeo.gwp.types.GqlZonedTime;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * Converts between protobuf {@code gql.Value} messages and native Java objects.
 *
 * <p>Protobuf types are referenced via the generated classes in the {@code gql} package,
 * which the protobuf-maven-plugin generates from the proto files.</p>
 */
public final class ValueConverter {

    private ValueConverter() {
        // utility class
    }

    // ========================================================================
    // Proto -> Native
    // ========================================================================

    /**
     * Convert a protobuf {@code Value} to a native Java object.
     *
     * <p>The mapping is:</p>
     * <ul>
     *   <li>{@code null_value} -> {@code null}</li>
     *   <li>{@code boolean_value} -> {@link Boolean}</li>
     *   <li>{@code integer_value} -> {@link Long}</li>
     *   <li>{@code unsigned_integer_value} -> {@link Long}</li>
     *   <li>{@code float_value} -> {@link Double}</li>
     *   <li>{@code string_value} -> {@link String}</li>
     *   <li>{@code bytes_value} -> {@code byte[]}</li>
     *   <li>{@code date_value} -> {@link GqlDate}</li>
     *   <li>{@code local_time_value} -> {@link GqlLocalTime}</li>
     *   <li>{@code zoned_time_value} -> {@link GqlZonedTime}</li>
     *   <li>{@code local_datetime_value} -> {@link GqlLocalDateTime}</li>
     *   <li>{@code zoned_datetime_value} -> {@link GqlZonedDateTime}</li>
     *   <li>{@code duration_value} -> {@link GqlDuration}</li>
     *   <li>{@code list_value} -> {@link List}</li>
     *   <li>{@code record_value} -> {@link GqlRecord}</li>
     *   <li>{@code node_value} -> {@link GqlNode}</li>
     *   <li>{@code edge_value} -> {@link GqlEdge}</li>
     *   <li>{@code path_value} -> {@link GqlPath}</li>
     * </ul>
     *
     * @param proto the protobuf Value message
     * @return the native Java value, or null for null/unrecognized types
     */
    public static Object fromProto(gql.GqlTypes.Value proto) {
        if (proto == null) {
            return null;
        }

        gql.GqlTypes.Value.KindCase kind = proto.getKindCase();

        return switch (kind) {
            case NULL_VALUE -> null;

            case BOOLEAN_VALUE -> proto.getBooleanValue();

            case INTEGER_VALUE -> proto.getIntegerValue();

            case UNSIGNED_INTEGER_VALUE -> proto.getUnsignedIntegerValue();

            case FLOAT_VALUE -> proto.getFloatValue();

            case STRING_VALUE -> proto.getStringValue();

            case BYTES_VALUE -> proto.getBytesValue().toByteArray();

            case DATE_VALUE -> {
                gql.GqlTypes.Date d = proto.getDateValue();
                yield new GqlDate(d.getYear(), d.getMonth(), d.getDay());
            }

            case LOCAL_TIME_VALUE -> {
                gql.GqlTypes.LocalTime t = proto.getLocalTimeValue();
                yield new GqlLocalTime(
                        t.getHour(), t.getMinute(), t.getSecond(), t.getNanosecond());
            }

            case ZONED_TIME_VALUE -> {
                gql.GqlTypes.ZonedTime zt = proto.getZonedTimeValue();
                gql.GqlTypes.LocalTime t = zt.getTime();
                yield new GqlZonedTime(
                        new GqlLocalTime(
                                t.getHour(), t.getMinute(), t.getSecond(), t.getNanosecond()),
                        zt.getOffsetMinutes());
            }

            case LOCAL_DATETIME_VALUE -> {
                gql.GqlTypes.LocalDateTime ldt = proto.getLocalDatetimeValue();
                gql.GqlTypes.Date d = ldt.getDate();
                gql.GqlTypes.LocalTime t = ldt.getTime();
                yield new GqlLocalDateTime(
                        new GqlDate(d.getYear(), d.getMonth(), d.getDay()),
                        new GqlLocalTime(
                                t.getHour(), t.getMinute(), t.getSecond(), t.getNanosecond()));
            }

            case ZONED_DATETIME_VALUE -> {
                gql.GqlTypes.ZonedDateTime zdt = proto.getZonedDatetimeValue();
                gql.GqlTypes.Date d = zdt.getDate();
                gql.GqlTypes.LocalTime t = zdt.getTime();
                yield new GqlZonedDateTime(
                        new GqlDate(d.getYear(), d.getMonth(), d.getDay()),
                        new GqlLocalTime(
                                t.getHour(), t.getMinute(), t.getSecond(), t.getNanosecond()),
                        zdt.getOffsetMinutes());
            }

            case DURATION_VALUE -> {
                gql.GqlTypes.Duration dur = proto.getDurationValue();
                yield new GqlDuration(dur.getMonths(), dur.getNanoseconds());
            }

            case LIST_VALUE -> {
                gql.GqlTypes.GqlList list = proto.getListValue();
                List<Object> result = new ArrayList<>(list.getElementsCount());
                for (gql.GqlTypes.Value elem : list.getElementsList()) {
                    result.add(fromProto(elem));
                }
                yield result;
            }

            case RECORD_VALUE -> {
                gql.GqlTypes.Record rec = proto.getRecordValue();
                List<GqlField> fields = new ArrayList<>(rec.getFieldsCount());
                for (gql.GqlTypes.Field f : rec.getFieldsList()) {
                    fields.add(new GqlField(f.getName(), fromProto(f.getValue())));
                }
                yield new GqlRecord(List.copyOf(fields));
            }

            case NODE_VALUE -> {
                gql.GqlTypes.Node n = proto.getNodeValue();
                Map<String, Object> props = convertProperties(n.getPropertiesMap());
                yield new GqlNode(
                        n.getId().toByteArray(),
                        List.copyOf(n.getLabelsList()),
                        props);
            }

            case EDGE_VALUE -> {
                gql.GqlTypes.Edge e = proto.getEdgeValue();
                Map<String, Object> props = convertProperties(e.getPropertiesMap());
                yield new GqlEdge(
                        e.getId().toByteArray(),
                        List.copyOf(e.getLabelsList()),
                        e.getSourceNodeId().toByteArray(),
                        e.getTargetNodeId().toByteArray(),
                        e.getUndirected(),
                        props);
            }

            case PATH_VALUE -> {
                gql.GqlTypes.Path p = proto.getPathValue();
                List<GqlNode> nodes = new ArrayList<>(p.getNodesCount());
                for (gql.GqlTypes.Node n : p.getNodesList()) {
                    Map<String, Object> props = convertProperties(n.getPropertiesMap());
                    nodes.add(new GqlNode(
                            n.getId().toByteArray(),
                            List.copyOf(n.getLabelsList()),
                            props));
                }
                List<GqlEdge> edges = new ArrayList<>(p.getEdgesCount());
                for (gql.GqlTypes.Edge e : p.getEdgesList()) {
                    Map<String, Object> props = convertProperties(e.getPropertiesMap());
                    edges.add(new GqlEdge(
                            e.getId().toByteArray(),
                            List.copyOf(e.getLabelsList()),
                            e.getSourceNodeId().toByteArray(),
                            e.getTargetNodeId().toByteArray(),
                            e.getUndirected(),
                            props));
                }
                yield new GqlPath(List.copyOf(nodes), List.copyOf(edges));
            }

            // BigInteger, BigFloat, Decimal - not supported in v0.1
            default -> null;
        };
    }

    // ========================================================================
    // Native -> Proto
    // ========================================================================

    /**
     * Convert a native Java value to a protobuf {@code Value}.
     *
     * <p>Supported types: {@code null}, {@link Boolean}, {@link Integer},
     * {@link Long}, {@link Float}, {@link Double}, {@link String},
     * {@code byte[]}, {@link GqlDate}, {@link GqlLocalTime},
     * {@link GqlZonedTime}, {@link GqlLocalDateTime}, {@link GqlZonedDateTime},
     * {@link GqlDuration}, {@link List}. Unsupported types map to null.</p>
     *
     * @param value the Java value to convert
     * @return the protobuf Value message
     */
    public static gql.GqlTypes.Value toProto(Object value) {
        if (value == null) {
            return gql.GqlTypes.Value.newBuilder()
                    .setNullValue(gql.GqlTypes.NullValue.getDefaultInstance())
                    .build();
        }

        if (value instanceof Boolean b) {
            return gql.GqlTypes.Value.newBuilder()
                    .setBooleanValue(b)
                    .build();
        }

        if (value instanceof Integer i) {
            return gql.GqlTypes.Value.newBuilder()
                    .setIntegerValue(i.longValue())
                    .build();
        }

        if (value instanceof Long l) {
            return gql.GqlTypes.Value.newBuilder()
                    .setIntegerValue(l)
                    .build();
        }

        if (value instanceof Float f) {
            return gql.GqlTypes.Value.newBuilder()
                    .setFloatValue(f.doubleValue())
                    .build();
        }

        if (value instanceof Double d) {
            return gql.GqlTypes.Value.newBuilder()
                    .setFloatValue(d)
                    .build();
        }

        if (value instanceof String s) {
            return gql.GqlTypes.Value.newBuilder()
                    .setStringValue(s)
                    .build();
        }

        if (value instanceof byte[] bytes) {
            return gql.GqlTypes.Value.newBuilder()
                    .setBytesValue(ByteString.copyFrom(bytes))
                    .build();
        }

        if (value instanceof GqlDate date) {
            return gql.GqlTypes.Value.newBuilder()
                    .setDateValue(gql.GqlTypes.Date.newBuilder()
                            .setYear(date.year())
                            .setMonth(date.month())
                            .setDay(date.day())
                            .build())
                    .build();
        }

        if (value instanceof GqlLocalTime time) {
            return gql.GqlTypes.Value.newBuilder()
                    .setLocalTimeValue(gql.GqlTypes.LocalTime.newBuilder()
                            .setHour(time.hour())
                            .setMinute(time.minute())
                            .setSecond(time.second())
                            .setNanosecond(time.nanosecond())
                            .build())
                    .build();
        }

        if (value instanceof GqlZonedTime zt) {
            GqlLocalTime t = zt.time();
            return gql.GqlTypes.Value.newBuilder()
                    .setZonedTimeValue(gql.GqlTypes.ZonedTime.newBuilder()
                            .setTime(gql.GqlTypes.LocalTime.newBuilder()
                                    .setHour(t.hour())
                                    .setMinute(t.minute())
                                    .setSecond(t.second())
                                    .setNanosecond(t.nanosecond())
                                    .build())
                            .setOffsetMinutes(zt.offsetMinutes())
                            .build())
                    .build();
        }

        if (value instanceof GqlLocalDateTime ldt) {
            GqlDate d = ldt.date();
            GqlLocalTime t = ldt.time();
            return gql.GqlTypes.Value.newBuilder()
                    .setLocalDatetimeValue(gql.GqlTypes.LocalDateTime.newBuilder()
                            .setDate(gql.GqlTypes.Date.newBuilder()
                                    .setYear(d.year())
                                    .setMonth(d.month())
                                    .setDay(d.day())
                                    .build())
                            .setTime(gql.GqlTypes.LocalTime.newBuilder()
                                    .setHour(t.hour())
                                    .setMinute(t.minute())
                                    .setSecond(t.second())
                                    .setNanosecond(t.nanosecond())
                                    .build())
                            .build())
                    .build();
        }

        if (value instanceof GqlZonedDateTime zdt) {
            GqlDate d = zdt.date();
            GqlLocalTime t = zdt.time();
            return gql.GqlTypes.Value.newBuilder()
                    .setZonedDatetimeValue(gql.GqlTypes.ZonedDateTime.newBuilder()
                            .setDate(gql.GqlTypes.Date.newBuilder()
                                    .setYear(d.year())
                                    .setMonth(d.month())
                                    .setDay(d.day())
                                    .build())
                            .setTime(gql.GqlTypes.LocalTime.newBuilder()
                                    .setHour(t.hour())
                                    .setMinute(t.minute())
                                    .setSecond(t.second())
                                    .setNanosecond(t.nanosecond())
                                    .build())
                            .setOffsetMinutes(zdt.offsetMinutes())
                            .build())
                    .build();
        }

        if (value instanceof GqlDuration dur) {
            return gql.GqlTypes.Value.newBuilder()
                    .setDurationValue(gql.GqlTypes.Duration.newBuilder()
                            .setMonths(dur.months())
                            .setNanoseconds(dur.nanoseconds())
                            .build())
                    .build();
        }

        if (value instanceof List<?> list) {
            gql.GqlTypes.GqlList.Builder listBuilder = gql.GqlTypes.GqlList.newBuilder();
            for (Object elem : list) {
                listBuilder.addElements(toProto(elem));
            }
            return gql.GqlTypes.Value.newBuilder()
                    .setListValue(listBuilder.build())
                    .build();
        }

        // Unsupported type: map to null
        return gql.GqlTypes.Value.newBuilder()
                .setNullValue(gql.GqlTypes.NullValue.getDefaultInstance())
                .build();
    }

    // ========================================================================
    // Internal helpers
    // ========================================================================

    private static Map<String, Object> convertProperties(
            Map<String, gql.GqlTypes.Value> protoMap) {
        Map<String, Object> result = new HashMap<>(protoMap.size());
        for (Map.Entry<String, gql.GqlTypes.Value> entry : protoMap.entrySet()) {
            result.put(entry.getKey(), fromProto(entry.getValue()));
        }
        return Map.copyOf(result);
    }
}
