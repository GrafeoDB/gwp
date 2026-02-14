package gwp

import (
	pb "github.com/GrafeoDB/gql-wire-protocol/go/gen/gql"
)

// valueFromProto converts a protobuf Value to a native Go value.
func valueFromProto(v *pb.Value) any {
	if v == nil {
		return nil
	}

	switch k := v.Kind.(type) {
	case *pb.Value_NullValue:
		return nil
	case *pb.Value_BooleanValue:
		return k.BooleanValue
	case *pb.Value_IntegerValue:
		return k.IntegerValue
	case *pb.Value_UnsignedIntegerValue:
		return k.UnsignedIntegerValue
	case *pb.Value_FloatValue:
		return k.FloatValue
	case *pb.Value_StringValue:
		return k.StringValue
	case *pb.Value_BytesValue:
		return k.BytesValue
	case *pb.Value_DateValue:
		d := k.DateValue
		return &GqlDate{Year: d.Year, Month: d.Month, Day: d.Day}
	case *pb.Value_LocalTimeValue:
		t := k.LocalTimeValue
		return &GqlLocalTime{
			Hour: t.Hour, Minute: t.Minute,
			Second: t.Second, Nanosecond: t.Nanosecond,
		}
	case *pb.Value_ZonedTimeValue:
		zt := k.ZonedTimeValue
		t := zt.Time
		return &GqlZonedTime{
			Time: GqlLocalTime{
				Hour: t.Hour, Minute: t.Minute,
				Second: t.Second, Nanosecond: t.Nanosecond,
			},
			OffsetMinutes: zt.OffsetMinutes,
		}
	case *pb.Value_LocalDatetimeValue:
		ldt := k.LocalDatetimeValue
		d := ldt.Date
		t := ldt.Time
		return &GqlLocalDateTime{
			Date: GqlDate{Year: d.Year, Month: d.Month, Day: d.Day},
			Time: GqlLocalTime{
				Hour: t.Hour, Minute: t.Minute,
				Second: t.Second, Nanosecond: t.Nanosecond,
			},
		}
	case *pb.Value_ZonedDatetimeValue:
		zdt := k.ZonedDatetimeValue
		d := zdt.Date
		t := zdt.Time
		return &GqlZonedDateTime{
			Date: GqlDate{Year: d.Year, Month: d.Month, Day: d.Day},
			Time: GqlLocalTime{
				Hour: t.Hour, Minute: t.Minute,
				Second: t.Second, Nanosecond: t.Nanosecond,
			},
			OffsetMinutes: zdt.OffsetMinutes,
		}
	case *pb.Value_DurationValue:
		dur := k.DurationValue
		return &GqlDuration{Months: dur.Months, Nanoseconds: dur.Nanoseconds}
	case *pb.Value_ListValue:
		elems := make([]any, len(k.ListValue.Elements))
		for i, e := range k.ListValue.Elements {
			elems[i] = valueFromProto(e)
		}
		return elems
	case *pb.Value_RecordValue:
		fields := make([]GqlField, len(k.RecordValue.Fields))
		for i, f := range k.RecordValue.Fields {
			fields[i] = GqlField{Name: f.Name, Value: valueFromProto(f.Value)}
		}
		return &GqlRecord{Fields: fields}
	case *pb.Value_NodeValue:
		n := k.NodeValue
		props := make(map[string]any, len(n.Properties))
		for key, pv := range n.Properties {
			props[key] = valueFromProto(pv)
		}
		return &GqlNode{ID: n.Id, Labels: n.Labels, Properties: props}
	case *pb.Value_EdgeValue:
		e := k.EdgeValue
		props := make(map[string]any, len(e.Properties))
		for key, pv := range e.Properties {
			props[key] = valueFromProto(pv)
		}
		return &GqlEdge{
			ID: e.Id, Labels: e.Labels,
			SourceNodeID: e.SourceNodeId, TargetNodeID: e.TargetNodeId,
			Undirected: e.Undirected, Properties: props,
		}
	case *pb.Value_PathValue:
		p := k.PathValue
		nodes := make([]*GqlNode, len(p.Nodes))
		for i, n := range p.Nodes {
			props := make(map[string]any, len(n.Properties))
			for key, pv := range n.Properties {
				props[key] = valueFromProto(pv)
			}
			nodes[i] = &GqlNode{ID: n.Id, Labels: n.Labels, Properties: props}
		}
		edges := make([]*GqlEdge, len(p.Edges))
		for i, e := range p.Edges {
			props := make(map[string]any, len(e.Properties))
			for key, pv := range e.Properties {
				props[key] = valueFromProto(pv)
			}
			edges[i] = &GqlEdge{
				ID: e.Id, Labels: e.Labels,
				SourceNodeID: e.SourceNodeId, TargetNodeID: e.TargetNodeId,
				Undirected: e.Undirected, Properties: props,
			}
		}
		return &GqlPath{Nodes: nodes, Edges: edges}
	default:
		return nil
	}
}

// valueToProto converts a native Go value to a protobuf Value.
func valueToProto(value any) *pb.Value {
	if value == nil {
		return &pb.Value{Kind: &pb.Value_NullValue{NullValue: &pb.NullValue{}}}
	}

	switch v := value.(type) {
	case bool:
		return &pb.Value{Kind: &pb.Value_BooleanValue{BooleanValue: v}}
	case int64:
		return &pb.Value{Kind: &pb.Value_IntegerValue{IntegerValue: v}}
	case int:
		return &pb.Value{Kind: &pb.Value_IntegerValue{IntegerValue: int64(v)}}
	case float64:
		return &pb.Value{Kind: &pb.Value_FloatValue{FloatValue: v}}
	case string:
		return &pb.Value{Kind: &pb.Value_StringValue{StringValue: v}}
	case []byte:
		return &pb.Value{Kind: &pb.Value_BytesValue{BytesValue: v}}
	case []any:
		elems := make([]*pb.Value, len(v))
		for i, e := range v {
			elems[i] = valueToProto(e)
		}
		return &pb.Value{Kind: &pb.Value_ListValue{ListValue: &pb.GqlList{Elements: elems}}}
	default:
		return &pb.Value{Kind: &pb.Value_NullValue{NullValue: &pb.NullValue{}}}
	}
}
