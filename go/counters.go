package gwp

// Write counter keys in ResultSummary.counters. A missing key counts as 0.
const (
	CounterNodesCreated  = "nodes_created"
	CounterNodesDeleted  = "nodes_deleted"
	CounterEdgesCreated  = "edges_created"
	CounterEdgesDeleted  = "edges_deleted"
	CounterPropertiesSet = "properties_set"
	CounterLabelsAdded   = "labels_added"
	CounterLabelsRemoved = "labels_removed"
)

// CounterKeys lists every write counter key, in field order.
var CounterKeys = []string{
	CounterNodesCreated,
	CounterNodesDeleted,
	CounterEdgesCreated,
	CounterEdgesDeleted,
	CounterPropertiesSet,
	CounterLabelsAdded,
	CounterLabelsRemoved,
}

// Counters are the write counters of a statement, read from the summary's
// counter map. Other entries of that map (such as execution_time_ms) are not
// write counters and are left out.
type Counters struct {
	NodesCreated  uint64
	NodesDeleted  uint64
	EdgesCreated  uint64
	EdgesDeleted  uint64
	PropertiesSet uint64
	LabelsAdded   uint64
	LabelsRemoved uint64
}

// CountersFromMap reads the write counters from a summary's counter map.
// Missing keys read as 0, and so does a negative value.
func CountersFromMap(m map[string]int64) Counters {
	get := func(key string) uint64 {
		if v := m[key]; v > 0 {
			return uint64(v)
		}
		return 0
	}
	return Counters{
		NodesCreated:  get(CounterNodesCreated),
		NodesDeleted:  get(CounterNodesDeleted),
		EdgesCreated:  get(CounterEdgesCreated),
		EdgesDeleted:  get(CounterEdgesDeleted),
		PropertiesSet: get(CounterPropertiesSet),
		LabelsAdded:   get(CounterLabelsAdded),
		LabelsRemoved: get(CounterLabelsRemoved),
	}
}

// ContainsUpdates reports whether any counter is non-zero, that is, whether
// the statement changed the graph.
func (c Counters) ContainsUpdates() bool {
	return c.NodesCreated > 0 || c.NodesDeleted > 0 || c.EdgesCreated > 0 ||
		c.EdgesDeleted > 0 || c.PropertiesSet > 0 || c.LabelsAdded > 0 ||
		c.LabelsRemoved > 0
}
