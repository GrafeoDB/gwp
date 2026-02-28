package gwp

import (
	"context"

	pb "github.com/GrafeoDB/gql-wire-protocol/go/gen/gql"
	"google.golang.org/grpc"
)

// SchemaInfo holds summary information about a schema.
type SchemaInfo struct {
	Name           string
	GraphCount     uint32
	GraphTypeCount uint32
}

// GraphInfo holds summary information about a graph.
type GraphInfo struct {
	Schema           string
	Name             string
	NodeCount        uint64
	EdgeCount        uint64
	GraphType        string
	StorageMode      string
	MemoryLimitBytes uint64
	BackwardEdges    bool
	Threads          uint32
}

// GraphTypeInfo holds summary information about a graph type.
type GraphTypeInfo struct {
	Schema string
	Name   string
}

// CreateGraphConfig holds configuration for creating a new graph.
type CreateGraphConfig struct {
	Schema           string
	Name             string
	IfNotExists      bool
	OrReplace        bool
	StorageMode      string
	MemoryLimitBytes *uint64
	BackwardEdges    *bool
	Threads          *uint32
	WalEnabled       *bool
	WalDurability    *string
}

// CatalogClient manages schemas, graphs, and graph types on a GWP server.
type CatalogClient struct {
	client pb.CatalogServiceClient
}

// NewCatalogClient creates a new CatalogClient from an existing gRPC connection.
func NewCatalogClient(conn *grpc.ClientConn) *CatalogClient {
	return &CatalogClient{
		client: pb.NewCatalogServiceClient(conn),
	}
}

// ListSchemas returns all schemas on the server.
func (c *CatalogClient) ListSchemas(ctx context.Context) ([]SchemaInfo, error) {
	resp, err := c.client.ListSchemas(ctx, &pb.ListSchemasRequest{})
	if err != nil {
		return nil, err
	}
	result := make([]SchemaInfo, len(resp.Schemas))
	for i, s := range resp.Schemas {
		result[i] = SchemaInfo{
			Name:           s.Name,
			GraphCount:     s.GraphCount,
			GraphTypeCount: s.GraphTypeCount,
		}
	}
	return result, nil
}

// CreateSchema creates a new schema.
func (c *CatalogClient) CreateSchema(ctx context.Context, name string, ifNotExists bool) error {
	_, err := c.client.CreateSchema(ctx, &pb.CreateSchemaRequest{
		Name:        name,
		IfNotExists: ifNotExists,
	})
	return err
}

// DropSchema drops a schema. Returns true if it existed.
func (c *CatalogClient) DropSchema(ctx context.Context, name string, ifExists bool) (bool, error) {
	resp, err := c.client.DropSchema(ctx, &pb.DropSchemaRequest{
		Name:     name,
		IfExists: ifExists,
	})
	if err != nil {
		return false, err
	}
	return resp.Existed, nil
}

// ListGraphs returns all graphs in a schema.
func (c *CatalogClient) ListGraphs(ctx context.Context, schema string) ([]GraphInfo, error) {
	resp, err := c.client.ListGraphs(ctx, &pb.ListGraphsRequest{
		Schema: schema,
	})
	if err != nil {
		return nil, err
	}
	result := make([]GraphInfo, len(resp.Graphs))
	for i, g := range resp.Graphs {
		result[i] = GraphInfo{
			Schema:    g.Schema,
			Name:      g.Name,
			NodeCount: g.NodeCount,
			EdgeCount: g.EdgeCount,
			GraphType: g.GraphType,
		}
	}
	return result, nil
}

// CreateGraph creates a new graph with the given configuration.
func (c *CatalogClient) CreateGraph(ctx context.Context, config CreateGraphConfig) (*GraphInfo, error) {
	opts := &pb.GraphOptions{}
	if config.MemoryLimitBytes != nil {
		opts.MemoryLimitBytes = config.MemoryLimitBytes
	}
	if config.BackwardEdges != nil {
		opts.BackwardEdges = config.BackwardEdges
	}
	if config.Threads != nil {
		opts.Threads = config.Threads
	}
	if config.WalEnabled != nil {
		opts.WalEnabled = config.WalEnabled
	}
	if config.WalDurability != nil {
		opts.WalDurability = config.WalDurability
	}

	resp, err := c.client.CreateGraph(ctx, &pb.CreateGraphRequest{
		Schema:      config.Schema,
		Name:        config.Name,
		IfNotExists: config.IfNotExists,
		OrReplace:   config.OrReplace,
		StorageMode: config.StorageMode,
		Options:     opts,
	})
	if err != nil {
		return nil, err
	}
	g := resp.Graph
	return &GraphInfo{
		Schema:    g.Schema,
		Name:      g.Name,
		NodeCount: g.NodeCount,
		EdgeCount: g.EdgeCount,
		GraphType: g.GraphType,
	}, nil
}

// DropGraph drops a graph. Returns true if it existed.
func (c *CatalogClient) DropGraph(ctx context.Context, schema, name string, ifExists bool) (bool, error) {
	resp, err := c.client.DropGraph(ctx, &pb.DropGraphRequest{
		Schema:   schema,
		Name:     name,
		IfExists: ifExists,
	})
	if err != nil {
		return false, err
	}
	return resp.Existed, nil
}

// GetGraphInfo returns detailed information about a specific graph.
func (c *CatalogClient) GetGraphInfo(ctx context.Context, schema, name string) (*GraphInfo, error) {
	resp, err := c.client.GetGraphInfo(ctx, &pb.GetGraphInfoRequest{
		Schema: schema,
		Name:   name,
	})
	if err != nil {
		return nil, err
	}
	return &GraphInfo{
		Schema:           resp.Schema,
		Name:             resp.Name,
		NodeCount:        resp.NodeCount,
		EdgeCount:        resp.EdgeCount,
		GraphType:        resp.GraphType,
		StorageMode:      resp.StorageMode,
		MemoryLimitBytes: resp.MemoryLimitBytes,
		BackwardEdges:    resp.BackwardEdges,
		Threads:          resp.Threads,
	}, nil
}

// ListGraphTypes returns all graph types in a schema.
func (c *CatalogClient) ListGraphTypes(ctx context.Context, schema string) ([]GraphTypeInfo, error) {
	resp, err := c.client.ListGraphTypes(ctx, &pb.ListGraphTypesRequest{
		Schema: schema,
	})
	if err != nil {
		return nil, err
	}
	result := make([]GraphTypeInfo, len(resp.GraphTypes))
	for i, t := range resp.GraphTypes {
		result[i] = GraphTypeInfo{
			Schema: t.Schema,
			Name:   t.Name,
		}
	}
	return result, nil
}

// CreateGraphType creates a new graph type.
func (c *CatalogClient) CreateGraphType(ctx context.Context, schema, name string, ifNotExists, orReplace bool) error {
	_, err := c.client.CreateGraphType(ctx, &pb.CreateGraphTypeRequest{
		Schema:      schema,
		Name:        name,
		IfNotExists: ifNotExists,
		OrReplace:   orReplace,
	})
	return err
}

// DropGraphType drops a graph type. Returns true if it existed.
func (c *CatalogClient) DropGraphType(ctx context.Context, schema, name string, ifExists bool) (bool, error) {
	resp, err := c.client.DropGraphType(ctx, &pb.DropGraphTypeRequest{
		Schema:   schema,
		Name:     name,
		IfExists: ifExists,
	})
	if err != nil {
		return false, err
	}
	return resp.Existed, nil
}
