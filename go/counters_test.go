package gwp

import "testing"

func TestCountersMissingKeysReadAsZero(t *testing.T) {
	c := CountersFromMap(nil)
	if c != (Counters{}) {
		t.Fatalf("expected zero counters, got %+v", c)
	}
	if c.ContainsUpdates() {
		t.Fatal("zero counters must not contain updates")
	}
}

func TestCountersReadEveryKeyAndIgnoreOthers(t *testing.T) {
	m := map[string]int64{"execution_time_ms": 99, "rows_scanned": 1000}
	for i, key := range CounterKeys {
		m[key] = int64(i + 1)
	}
	got := CountersFromMap(m)
	want := Counters{
		NodesCreated:  1,
		NodesDeleted:  2,
		EdgesCreated:  3,
		EdgesDeleted:  4,
		PropertiesSet: 5,
		LabelsAdded:   6,
		LabelsRemoved: 7,
	}
	if got != want {
		t.Fatalf("got %+v, want %+v", got, want)
	}
}

func TestCountersNegativeReadsAsZero(t *testing.T) {
	c := CountersFromMap(map[string]int64{CounterNodesCreated: -5})
	if c.NodesCreated != 0 {
		t.Fatalf("expected 0, got %d", c.NodesCreated)
	}
}

func TestCountersSingleKeyIsAnUpdate(t *testing.T) {
	for _, key := range CounterKeys {
		if !CountersFromMap(map[string]int64{key: 1}).ContainsUpdates() {
			t.Fatalf("%s=1 must count as an update", key)
		}
	}
}
