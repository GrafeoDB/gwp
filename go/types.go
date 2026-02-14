package gwp

// GqlNode is a property graph node.
type GqlNode struct {
	ID         []byte
	Labels     []string
	Properties map[string]any
}

// HasLabel checks if the node has the given label.
func (n *GqlNode) HasLabel(label string) bool {
	for _, l := range n.Labels {
		if l == label {
			return true
		}
	}
	return false
}

// GqlEdge is a property graph edge.
type GqlEdge struct {
	ID           []byte
	Labels       []string
	SourceNodeID []byte
	TargetNodeID []byte
	Undirected   bool
	Properties   map[string]any
}

// HasLabel checks if the edge has the given label.
func (e *GqlEdge) HasLabel(label string) bool {
	for _, l := range e.Labels {
		if l == label {
			return true
		}
	}
	return false
}

// GqlPath is an alternating sequence of nodes and edges.
type GqlPath struct {
	Nodes []*GqlNode
	Edges []*GqlEdge
}

// Len returns the number of edges in the path.
func (p *GqlPath) Len() int {
	return len(p.Edges)
}

// GqlRecord is a named collection of fields.
type GqlRecord struct {
	Fields []GqlField
}

// Get returns the value of the field with the given name, or nil if not found.
func (r *GqlRecord) Get(name string) any {
	for _, f := range r.Fields {
		if f.Name == name {
			return f.Value
		}
	}
	return nil
}

// GqlField is a single field in a record.
type GqlField struct {
	Name  string
	Value any
}

// GqlDate is a calendar date.
type GqlDate struct {
	Year  int32
	Month uint32
	Day   uint32
}

// GqlLocalTime is a time without timezone.
type GqlLocalTime struct {
	Hour       uint32
	Minute     uint32
	Second     uint32
	Nanosecond uint32
}

// GqlZonedTime is a time with UTC offset.
type GqlZonedTime struct {
	Time          GqlLocalTime
	OffsetMinutes int32
}

// GqlLocalDateTime is a date and time without timezone.
type GqlLocalDateTime struct {
	Date GqlDate
	Time GqlLocalTime
}

// GqlZonedDateTime is a date and time with UTC offset.
type GqlZonedDateTime struct {
	Date          GqlDate
	Time          GqlLocalTime
	OffsetMinutes int32
}

// GqlDuration is a temporal duration.
type GqlDuration struct {
	Months      int64
	Nanoseconds int64
}
