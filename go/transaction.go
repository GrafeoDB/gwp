package gwp

import (
	"context"

	pb "github.com/GrafeoDB/gql-wire-protocol/go/gen/gql"
)

// Transaction is an explicit transaction within a session.
type Transaction struct {
	sessionID     string
	transactionID string
	gqlClient     pb.GqlServiceClient
	committed     bool
	rolledBack    bool
}

// TransactionID returns the transaction identifier.
func (t *Transaction) TransactionID() string {
	return t.transactionID
}

// Execute executes a statement within this transaction.
func (t *Transaction) Execute(ctx context.Context, statement string, params map[string]any) (*ResultCursor, error) {
	protoParams := make(map[string]*pb.Value, len(params))
	for k, v := range params {
		protoParams[k] = valueToProto(v)
	}

	stream, err := t.gqlClient.Execute(ctx, &pb.ExecuteRequest{
		SessionId:     t.sessionID,
		Statement:     statement,
		Parameters:    protoParams,
		TransactionId: t.transactionID,
	})
	if err != nil {
		return nil, err
	}

	return newResultCursor(stream), nil
}

// Commit commits the transaction.
func (t *Transaction) Commit(ctx context.Context) error {
	resp, err := t.gqlClient.Commit(ctx, &pb.CommitRequest{
		SessionId:     t.sessionID,
		TransactionId: t.transactionID,
	})
	if err != nil {
		return err
	}
	t.committed = true

	if resp.Status != nil && IsException(resp.Status.Code) {
		return &GqlStatusError{Code: resp.Status.Code, Message: resp.Status.Message}
	}
	return nil
}

// Rollback rolls back the transaction. No-op after commit or previous rollback.
func (t *Transaction) Rollback(ctx context.Context) error {
	if t.committed || t.rolledBack {
		return nil
	}

	resp, err := t.gqlClient.Rollback(ctx, &pb.RollbackRequest{
		SessionId:     t.sessionID,
		TransactionId: t.transactionID,
	})
	if err != nil {
		return err
	}
	t.rolledBack = true

	if resp.Status != nil && IsException(resp.Status.Code) {
		return &GqlStatusError{Code: resp.Status.Code, Message: resp.Status.Message}
	}
	return nil
}
