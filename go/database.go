package gwp

import (
	"context"

	pb "github.com/GrafeoDB/gql-wire-protocol/go/gen/gql"
	"google.golang.org/grpc"
)

// DatabaseInfo holds summary information about a database.
type DatabaseInfo struct {
	Name             string
	NodeCount        uint64
	EdgeCount        uint64
	Persistent       bool
	DatabaseType     string
	StorageMode      string
	MemoryLimitBytes uint64
	BackwardEdges    bool
	Threads          uint32
}

// CreateDatabaseConfig holds configuration for creating a new database.
type CreateDatabaseConfig struct {
	Name             string
	DatabaseType     string
	StorageMode      string
	MemoryLimitBytes *uint64
	BackwardEdges    *bool
	Threads          *uint32
	WalEnabled       *bool
	WalDurability    *string
}

// DatabaseClient manages databases on a GWP server.
type DatabaseClient struct {
	client pb.DatabaseServiceClient
}

// NewDatabaseClient creates a new DatabaseClient from an existing gRPC connection.
func NewDatabaseClient(conn *grpc.ClientConn) *DatabaseClient {
	return &DatabaseClient{
		client: pb.NewDatabaseServiceClient(conn),
	}
}

// List returns all databases on the server.
func (d *DatabaseClient) List(ctx context.Context) ([]DatabaseInfo, error) {
	resp, err := d.client.ListDatabases(ctx, &pb.ListDatabasesRequest{})
	if err != nil {
		return nil, err
	}
	result := make([]DatabaseInfo, len(resp.Databases))
	for i, db := range resp.Databases {
		result[i] = DatabaseInfo{
			Name:         db.Name,
			NodeCount:    db.NodeCount,
			EdgeCount:    db.EdgeCount,
			Persistent:   db.Persistent,
			DatabaseType: db.DatabaseType,
		}
	}
	return result, nil
}

// Create creates a new database with the given configuration.
func (d *DatabaseClient) Create(ctx context.Context, config CreateDatabaseConfig) (*DatabaseInfo, error) {
	opts := &pb.DatabaseOptions{}
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

	resp, err := d.client.CreateDatabase(ctx, &pb.CreateDatabaseRequest{
		Name:         config.Name,
		DatabaseType: config.DatabaseType,
		StorageMode:  config.StorageMode,
		Options:      opts,
	})
	if err != nil {
		return nil, err
	}
	db := resp.Database
	return &DatabaseInfo{
		Name:         db.Name,
		NodeCount:    db.NodeCount,
		EdgeCount:    db.EdgeCount,
		Persistent:   db.Persistent,
		DatabaseType: db.DatabaseType,
	}, nil
}

// Delete deletes a database by name. Returns the name of the deleted database.
func (d *DatabaseClient) Delete(ctx context.Context, name string) (string, error) {
	resp, err := d.client.DeleteDatabase(ctx, &pb.DeleteDatabaseRequest{
		Name: name,
	})
	if err != nil {
		return "", err
	}
	return resp.Deleted, nil
}

// GetInfo returns detailed information about a specific database.
func (d *DatabaseClient) GetInfo(ctx context.Context, name string) (*DatabaseInfo, error) {
	resp, err := d.client.GetDatabaseInfo(ctx, &pb.GetDatabaseInfoRequest{
		Name: name,
	})
	if err != nil {
		return nil, err
	}
	return &DatabaseInfo{
		Name:             resp.Name,
		NodeCount:        resp.NodeCount,
		EdgeCount:        resp.EdgeCount,
		Persistent:       resp.Persistent,
		DatabaseType:     resp.DatabaseType,
		StorageMode:      resp.StorageMode,
		MemoryLimitBytes: resp.MemoryLimitBytes,
		BackwardEdges:    resp.BackwardEdges,
		Threads:          resp.Threads,
	}, nil
}
