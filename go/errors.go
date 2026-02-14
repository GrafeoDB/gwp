package gwp

import "fmt"

// GqlError is the base error type for GWP operations.
type GqlError struct {
	Message string
}

func (e *GqlError) Error() string {
	return e.Message
}

// GqlStatusError represents a GQL status error with a GQLSTATUS code.
type GqlStatusError struct {
	Code    string
	Message string
}

func (e *GqlStatusError) Error() string {
	return fmt.Sprintf("[%s] %s", e.Code, e.Message)
}

// SessionError represents a session-level error.
type SessionError struct {
	Message string
}

func (e *SessionError) Error() string {
	return e.Message
}

// TransactionError represents a transaction-level error.
type TransactionError struct {
	Message string
}

func (e *TransactionError) Error() string {
	return e.Message
}
