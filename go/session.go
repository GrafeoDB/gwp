package gwp

import (
	"context"
	"io"

	pb "github.com/GrafeoDB/gql-wire-protocol/go/gen/gql"
)

// GqlSession is an active session with a GWP server.
type GqlSession struct {
	sessionID     string
	sessionClient pb.SessionServiceClient
	gqlClient     pb.GqlServiceClient
	closed        bool
}

// SessionID returns the session identifier.
func (s *GqlSession) SessionID() string {
	return s.sessionID
}

// Execute executes a GQL statement and returns a result cursor.
func (s *GqlSession) Execute(ctx context.Context, statement string, params map[string]any) (*ResultCursor, error) {
	protoParams := make(map[string]*pb.Value, len(params))
	for k, v := range params {
		protoParams[k] = valueToProto(v)
	}

	stream, err := s.gqlClient.Execute(ctx, &pb.ExecuteRequest{
		SessionId:  s.sessionID,
		Statement:  statement,
		Parameters: protoParams,
	})
	if err != nil {
		return nil, err
	}

	return newResultCursor(stream), nil
}

// BeginTransaction begins a new explicit transaction.
func (s *GqlSession) BeginTransaction(ctx context.Context, readOnly bool) (*Transaction, error) {
	mode := pb.TransactionMode_READ_WRITE
	if readOnly {
		mode = pb.TransactionMode_READ_ONLY
	}

	resp, err := s.gqlClient.BeginTransaction(ctx, &pb.BeginRequest{
		SessionId: s.sessionID,
		Mode:      mode,
	})
	if err != nil {
		return nil, err
	}

	if resp.Status != nil && IsException(resp.Status.Code) {
		return nil, &GqlStatusError{Code: resp.Status.Code, Message: resp.Status.Message}
	}

	if resp.TransactionId == "" {
		return nil, &TransactionError{Message: "server returned empty transaction ID"}
	}

	return &Transaction{
		sessionID:     s.sessionID,
		transactionID: resp.TransactionId,
		gqlClient:     s.gqlClient,
	}, nil
}

// SetGraph sets the current graph for the session.
func (s *GqlSession) SetGraph(ctx context.Context, name string) error {
	_, err := s.sessionClient.Configure(ctx, &pb.ConfigureRequest{
		SessionId: s.sessionID,
		Property:  &pb.ConfigureRequest_Graph{Graph: name},
	})
	return err
}

// SetSchema sets the current schema for the session.
func (s *GqlSession) SetSchema(ctx context.Context, name string) error {
	_, err := s.sessionClient.Configure(ctx, &pb.ConfigureRequest{
		SessionId: s.sessionID,
		Property:  &pb.ConfigureRequest_Schema{Schema: name},
	})
	return err
}

// SetTimeZone sets the session timezone offset in minutes.
func (s *GqlSession) SetTimeZone(ctx context.Context, offsetMinutes int32) error {
	_, err := s.sessionClient.Configure(ctx, &pb.ConfigureRequest{
		SessionId: s.sessionID,
		Property:  &pb.ConfigureRequest_TimeZoneOffsetMinutes{TimeZoneOffsetMinutes: offsetMinutes},
	})
	return err
}

// Reset resets session state to defaults.
func (s *GqlSession) Reset(ctx context.Context) error {
	_, err := s.sessionClient.Reset(ctx, &pb.ResetRequest{
		SessionId: s.sessionID,
		Target:    pb.ResetTarget_RESET_ALL,
	})
	return err
}

// Ping pings the server and returns a timestamp.
func (s *GqlSession) Ping(ctx context.Context) (int64, error) {
	resp, err := s.sessionClient.Ping(ctx, &pb.PingRequest{
		SessionId: s.sessionID,
	})
	if err != nil {
		return 0, err
	}
	return resp.Timestamp, nil
}

// Close closes the session.
func (s *GqlSession) Close(ctx context.Context) error {
	if s.closed {
		return nil
	}
	_, err := s.sessionClient.Close(ctx, &pb.CloseRequest{
		SessionId: s.sessionID,
	})
	s.closed = true
	return err
}

// resultCursorStream is the interface for the gRPC stream.
type resultCursorStream interface {
	Recv() (*pb.ExecuteResponse, error)
}

func newResultCursor(stream resultCursorStream) *ResultCursor {
	return &ResultCursor{stream: stream}
}

// ResultCursor is a cursor over streaming result frames.
type ResultCursor struct {
	stream      resultCursorStream
	header      *pb.ResultHeader
	summary     *pb.ResultSummary
	bufferedRows [][]any
	rowIndex    int
	done        bool
}

func (c *ResultCursor) consumeUntilRowsOrDone() error {
	for !c.done && c.rowIndex >= len(c.bufferedRows) {
		resp, err := c.stream.Recv()
		if err == io.EOF {
			c.done = true
			return nil
		}
		if err != nil {
			c.done = true
			return err
		}

		switch f := resp.Frame.(type) {
		case *pb.ExecuteResponse_Header:
			c.header = f.Header
		case *pb.ExecuteResponse_RowBatch:
			for _, row := range f.RowBatch.Rows {
				values := make([]any, len(row.Values))
				for i, v := range row.Values {
					values[i] = valueFromProto(v)
				}
				c.bufferedRows = append(c.bufferedRows, values)
			}
		case *pb.ExecuteResponse_Summary:
			c.summary = f.Summary
			c.done = true
		}
	}
	return nil
}

// ColumnNames returns the column names from the result header.
func (c *ResultCursor) ColumnNames() ([]string, error) {
	if c.header == nil {
		if err := c.consumeUntilRowsOrDone(); err != nil {
			return nil, err
		}
	}
	if c.header == nil {
		return nil, nil
	}
	names := make([]string, len(c.header.Columns))
	for i, col := range c.header.Columns {
		names[i] = col.Name
	}
	return names, nil
}

// NextRow returns the next row, or nil when done.
func (c *ResultCursor) NextRow() ([]any, error) {
	if c.rowIndex < len(c.bufferedRows) {
		row := c.bufferedRows[c.rowIndex]
		c.rowIndex++
		return row, nil
	}

	if err := c.consumeUntilRowsOrDone(); err != nil {
		return nil, err
	}

	if c.rowIndex < len(c.bufferedRows) {
		row := c.bufferedRows[c.rowIndex]
		c.rowIndex++
		return row, nil
	}

	return nil, nil
}

// CollectRows collects all remaining rows.
func (c *ResultCursor) CollectRows() ([][]any, error) {
	var rows [][]any
	for {
		row, err := c.NextRow()
		if err != nil {
			return rows, err
		}
		if row == nil {
			return rows, nil
		}
		rows = append(rows, row)
	}
}

// Summary returns the result summary. Consumes remaining frames if needed.
func (c *ResultCursor) Summary() (*ResultSummary, error) {
	for !c.done {
		c.rowIndex = len(c.bufferedRows)
		if err := c.consumeUntilRowsOrDone(); err != nil {
			return nil, err
		}
	}
	if c.summary != nil {
		return &ResultSummary{proto: c.summary}, nil
	}
	return nil, nil
}

// IsSuccess checks if the execution was successful.
func (c *ResultCursor) IsSuccess() (bool, error) {
	s, err := c.Summary()
	if err != nil {
		return false, err
	}
	if s == nil {
		return false, nil
	}
	return s.IsSuccess(), nil
}

// RowsAffected returns the number of rows affected.
func (c *ResultCursor) RowsAffected() (int64, error) {
	s, err := c.Summary()
	if err != nil {
		return 0, err
	}
	if s == nil {
		return 0, nil
	}
	return s.RowsAffected(), nil
}

// ResultSummary wraps a protobuf result summary.
type ResultSummary struct {
	proto *pb.ResultSummary
}

// StatusCode returns the GQLSTATUS code.
func (s *ResultSummary) StatusCode() string {
	if s.proto.Status != nil {
		return s.proto.Status.Code
	}
	return ""
}

// Message returns the status message.
func (s *ResultSummary) Message() string {
	if s.proto.Status != nil {
		return s.proto.Status.Message
	}
	return ""
}

// RowsAffected returns the number of rows affected.
func (s *ResultSummary) RowsAffected() int64 {
	return s.proto.RowsAffected
}

// IsSuccess checks if the execution was successful.
func (s *ResultSummary) IsSuccess() bool {
	return IsSuccess(s.StatusCode())
}
