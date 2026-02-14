package gwp

import (
	"context"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	pb "github.com/GrafeoDB/gql-wire-protocol/go/gen/gql"
)

// GqlConnection is a connection to a GWP server.
type GqlConnection struct {
	conn          *grpc.ClientConn
	sessionClient pb.SessionServiceClient
	gqlClient     pb.GqlServiceClient
}

// Connect creates a new connection to a GWP server.
func Connect(ctx context.Context, target string, opts ...grpc.DialOption) (*GqlConnection, error) {
	if len(opts) == 0 {
		opts = append(opts, grpc.WithTransportCredentials(insecure.NewCredentials()))
	}

	conn, err := grpc.NewClient(target, opts...)
	if err != nil {
		return nil, &GqlError{Message: "failed to connect: " + err.Error()}
	}

	return &GqlConnection{
		conn:          conn,
		sessionClient: pb.NewSessionServiceClient(conn),
		gqlClient:     pb.NewGqlServiceClient(conn),
	}, nil
}

// CreateSession performs a handshake and returns a new session.
func (c *GqlConnection) CreateSession(ctx context.Context) (*GqlSession, error) {
	resp, err := c.sessionClient.Handshake(ctx, &pb.HandshakeRequest{
		ProtocolVersion: 1,
	})
	if err != nil {
		return nil, err
	}

	if resp.SessionId == "" {
		return nil, &SessionError{Message: "server returned empty session ID"}
	}

	return &GqlSession{
		sessionID:     resp.SessionId,
		sessionClient: c.sessionClient,
		gqlClient:     c.gqlClient,
	}, nil
}

// Close closes the underlying gRPC connection.
func (c *GqlConnection) Close() error {
	return c.conn.Close()
}
