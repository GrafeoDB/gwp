package gwp

import (
	"context"
	"fmt"
	"net"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"testing"
	"time"
)

var testEndpoint string

func TestMain(m *testing.M) {
	// Find the test server binary
	repoRoot := filepath.Join("..", "")
	binary := filepath.Join(repoRoot, "target", "release", "gwp-test-server")
	if runtime.GOOS == "windows" {
		binary += ".exe"
	}

	if _, err := os.Stat(binary); os.IsNotExist(err) {
		fmt.Fprintf(os.Stderr, "gwp-test-server not found at %s, skipping integration tests\n", binary)
		os.Exit(0)
	}

	// Find a free port
	l, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		fmt.Fprintf(os.Stderr, "failed to find free port: %v\n", err)
		os.Exit(1)
	}
	port := l.Addr().(*net.TCPAddr).Port
	l.Close()

	// Start the server
	cmd := exec.Command(binary, fmt.Sprintf("%d", port))
	cmd.Stdout = os.Stderr
	cmd.Stderr = os.Stderr
	if err := cmd.Start(); err != nil {
		fmt.Fprintf(os.Stderr, "failed to start test server: %v\n", err)
		os.Exit(1)
	}

	// Wait for server to be ready
	deadline := time.Now().Add(10 * time.Second)
	for time.Now().Before(deadline) {
		conn, err := net.DialTimeout("tcp", fmt.Sprintf("127.0.0.1:%d", port), 500*time.Millisecond)
		if err == nil {
			conn.Close()
			break
		}
		time.Sleep(100 * time.Millisecond)
	}

	testEndpoint = fmt.Sprintf("localhost:%d", port)

	code := m.Run()

	cmd.Process.Kill()
	cmd.Wait()

	os.Exit(code)
}

func TestConnectAndCreateSession(t *testing.T) {
	ctx := context.Background()
	conn, err := Connect(ctx, testEndpoint)
	if err != nil {
		t.Fatalf("Connect: %v", err)
	}
	defer conn.Close()

	session, err := conn.CreateSession(ctx)
	if err != nil {
		t.Fatalf("CreateSession: %v", err)
	}
	defer session.Close(ctx)

	if session.SessionID() == "" {
		t.Fatal("expected non-empty session ID")
	}
}

func TestPing(t *testing.T) {
	ctx := context.Background()
	conn, err := Connect(ctx, testEndpoint)
	if err != nil {
		t.Fatalf("Connect: %v", err)
	}
	defer conn.Close()

	session, err := conn.CreateSession(ctx)
	if err != nil {
		t.Fatalf("CreateSession: %v", err)
	}
	defer session.Close(ctx)

	ts, err := session.Ping(ctx)
	if err != nil {
		t.Fatalf("Ping: %v", err)
	}
	if ts <= 0 {
		t.Fatalf("expected positive timestamp, got %d", ts)
	}
}

func TestSetGraphSchemaTimeZone(t *testing.T) {
	ctx := context.Background()
	conn, err := Connect(ctx, testEndpoint)
	if err != nil {
		t.Fatalf("Connect: %v", err)
	}
	defer conn.Close()

	session, err := conn.CreateSession(ctx)
	if err != nil {
		t.Fatalf("CreateSession: %v", err)
	}
	defer session.Close(ctx)

	if err := session.SetGraph(ctx, "mygraph"); err != nil {
		t.Fatalf("SetGraph: %v", err)
	}
	if err := session.SetSchema(ctx, "myschema"); err != nil {
		t.Fatalf("SetSchema: %v", err)
	}
	if err := session.SetTimeZone(ctx, -300); err != nil {
		t.Fatalf("SetTimeZone: %v", err)
	}
	if err := session.Reset(ctx); err != nil {
		t.Fatalf("Reset: %v", err)
	}
}

func TestMatchQuery(t *testing.T) {
	ctx := context.Background()
	conn, err := Connect(ctx, testEndpoint)
	if err != nil {
		t.Fatalf("Connect: %v", err)
	}
	defer conn.Close()

	session, err := conn.CreateSession(ctx)
	if err != nil {
		t.Fatalf("CreateSession: %v", err)
	}
	defer session.Close(ctx)

	cursor, err := session.Execute(ctx, "MATCH (n:Person) RETURN n.name, n.age", nil)
	if err != nil {
		t.Fatalf("Execute: %v", err)
	}

	cols, err := cursor.ColumnNames()
	if err != nil {
		t.Fatalf("ColumnNames: %v", err)
	}
	if len(cols) != 2 || cols[0] != "name" || cols[1] != "age" {
		t.Fatalf("expected [name age], got %v", cols)
	}

	rows, err := cursor.CollectRows()
	if err != nil {
		t.Fatalf("CollectRows: %v", err)
	}
	if len(rows) != 2 {
		t.Fatalf("expected 2 rows, got %d", len(rows))
	}
	if rows[0][0] != "Alice" {
		t.Fatalf("expected Alice, got %v", rows[0][0])
	}
	if rows[0][1] != int64(30) {
		t.Fatalf("expected 30, got %v", rows[0][1])
	}
}

func TestDDLOmittedResult(t *testing.T) {
	ctx := context.Background()
	conn, err := Connect(ctx, testEndpoint)
	if err != nil {
		t.Fatalf("Connect: %v", err)
	}
	defer conn.Close()

	session, err := conn.CreateSession(ctx)
	if err != nil {
		t.Fatalf("CreateSession: %v", err)
	}
	defer session.Close(ctx)

	cursor, err := session.Execute(ctx, "CREATE GRAPH mygraph", nil)
	if err != nil {
		t.Fatalf("Execute: %v", err)
	}

	rows, err := cursor.CollectRows()
	if err != nil {
		t.Fatalf("CollectRows: %v", err)
	}
	if len(rows) != 0 {
		t.Fatalf("expected 0 rows, got %d", len(rows))
	}
}

func TestDMLRowsAffected(t *testing.T) {
	ctx := context.Background()
	conn, err := Connect(ctx, testEndpoint)
	if err != nil {
		t.Fatalf("Connect: %v", err)
	}
	defer conn.Close()

	session, err := conn.CreateSession(ctx)
	if err != nil {
		t.Fatalf("CreateSession: %v", err)
	}
	defer session.Close(ctx)

	cursor, err := session.Execute(ctx, "INSERT INTO t VALUES (1)", nil)
	if err != nil {
		t.Fatalf("Execute: %v", err)
	}

	_, err = cursor.CollectRows()
	if err != nil {
		t.Fatalf("CollectRows: %v", err)
	}

	affected, err := cursor.RowsAffected()
	if err != nil {
		t.Fatalf("RowsAffected: %v", err)
	}
	if affected != 3 {
		t.Fatalf("expected 3 rows affected, got %d", affected)
	}
}

func TestIsSuccessOnMatch(t *testing.T) {
	ctx := context.Background()
	conn, err := Connect(ctx, testEndpoint)
	if err != nil {
		t.Fatalf("Connect: %v", err)
	}
	defer conn.Close()

	session, err := conn.CreateSession(ctx)
	if err != nil {
		t.Fatalf("CreateSession: %v", err)
	}
	defer session.Close(ctx)

	cursor, err := session.Execute(ctx, "MATCH (n) RETURN n", nil)
	if err != nil {
		t.Fatalf("Execute: %v", err)
	}

	ok, err := cursor.IsSuccess()
	if err != nil {
		t.Fatalf("IsSuccess: %v", err)
	}
	if !ok {
		t.Fatal("expected success")
	}
}

func TestTransactionCommit(t *testing.T) {
	ctx := context.Background()
	conn, err := Connect(ctx, testEndpoint)
	if err != nil {
		t.Fatalf("Connect: %v", err)
	}
	defer conn.Close()

	session, err := conn.CreateSession(ctx)
	if err != nil {
		t.Fatalf("CreateSession: %v", err)
	}
	defer session.Close(ctx)

	tx, err := session.BeginTransaction(ctx, false)
	if err != nil {
		t.Fatalf("BeginTransaction: %v", err)
	}

	cursor, err := tx.Execute(ctx, "INSERT INTO t VALUES (1)", nil)
	if err != nil {
		t.Fatalf("Execute: %v", err)
	}
	_, _ = cursor.CollectRows()

	if err := tx.Commit(ctx); err != nil {
		t.Fatalf("Commit: %v", err)
	}
}

func TestTransactionRollback(t *testing.T) {
	ctx := context.Background()
	conn, err := Connect(ctx, testEndpoint)
	if err != nil {
		t.Fatalf("Connect: %v", err)
	}
	defer conn.Close()

	session, err := conn.CreateSession(ctx)
	if err != nil {
		t.Fatalf("CreateSession: %v", err)
	}
	defer session.Close(ctx)

	tx, err := session.BeginTransaction(ctx, false)
	if err != nil {
		t.Fatalf("BeginTransaction: %v", err)
	}

	cursor, err := tx.Execute(ctx, "INSERT INTO t VALUES (1)", nil)
	if err != nil {
		t.Fatalf("Execute: %v", err)
	}
	_, _ = cursor.CollectRows()

	if err := tx.Rollback(ctx); err != nil {
		t.Fatalf("Rollback: %v", err)
	}
}

func TestTransactionMatchQuery(t *testing.T) {
	ctx := context.Background()
	conn, err := Connect(ctx, testEndpoint)
	if err != nil {
		t.Fatalf("Connect: %v", err)
	}
	defer conn.Close()

	session, err := conn.CreateSession(ctx)
	if err != nil {
		t.Fatalf("CreateSession: %v", err)
	}
	defer session.Close(ctx)

	tx, err := session.BeginTransaction(ctx, false)
	if err != nil {
		t.Fatalf("BeginTransaction: %v", err)
	}

	cursor, err := tx.Execute(ctx, "MATCH (n:Person) RETURN n.name", nil)
	if err != nil {
		t.Fatalf("Execute: %v", err)
	}

	rows, err := cursor.CollectRows()
	if err != nil {
		t.Fatalf("CollectRows: %v", err)
	}
	if len(rows) != 2 {
		t.Fatalf("expected 2 rows, got %d", len(rows))
	}

	if err := tx.Commit(ctx); err != nil {
		t.Fatalf("Commit: %v", err)
	}
}

func TestRollbackAfterCommit(t *testing.T) {
	ctx := context.Background()
	conn, err := Connect(ctx, testEndpoint)
	if err != nil {
		t.Fatalf("Connect: %v", err)
	}
	defer conn.Close()

	session, err := conn.CreateSession(ctx)
	if err != nil {
		t.Fatalf("CreateSession: %v", err)
	}
	defer session.Close(ctx)

	tx, err := session.BeginTransaction(ctx, false)
	if err != nil {
		t.Fatalf("BeginTransaction: %v", err)
	}

	if err := tx.Commit(ctx); err != nil {
		t.Fatalf("Commit: %v", err)
	}

	// Rollback after commit should be a no-op
	if err := tx.Rollback(ctx); err != nil {
		t.Fatalf("Rollback after commit: %v", err)
	}
}
