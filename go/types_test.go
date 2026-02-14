package gwp

import "testing"

func TestNodeHasLabel(t *testing.T) {
	node := &GqlNode{
		ID:     []byte{1},
		Labels: []string{"Person"},
		Properties: map[string]any{
			"name": "Alice",
		},
	}
	if !node.HasLabel("Person") {
		t.Fatal("expected has label Person")
	}
	if node.HasLabel("Company") {
		t.Fatal("expected does not have label Company")
	}
}

func TestEdgeHasLabel(t *testing.T) {
	edge := &GqlEdge{
		ID:           []byte{16},
		Labels:       []string{"knows"},
		SourceNodeID: []byte{1},
		TargetNodeID: []byte{2},
	}
	if !edge.HasLabel("knows") {
		t.Fatal("expected has label knows")
	}
}

func TestPathLen(t *testing.T) {
	a := &GqlNode{ID: []byte{1}, Labels: []string{"A"}}
	b := &GqlNode{ID: []byte{2}, Labels: []string{"B"}}
	e := &GqlEdge{ID: []byte{16}, Labels: []string{"to"}, SourceNodeID: []byte{1}, TargetNodeID: []byte{2}}
	path := &GqlPath{Nodes: []*GqlNode{a, b}, Edges: []*GqlEdge{e}}
	if path.Len() != 1 {
		t.Fatalf("expected path length 1, got %d", path.Len())
	}
}

func TestRecordGet(t *testing.T) {
	rec := &GqlRecord{
		Fields: []GqlField{
			{Name: "x", Value: int64(1)},
			{Name: "y", Value: int64(2)},
		},
	}
	if rec.Get("x") != int64(1) {
		t.Fatal("expected x=1")
	}
	if rec.Get("z") != nil {
		t.Fatal("expected z=nil")
	}
}
