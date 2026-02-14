# GQL Wire Protocol - Java Binding

Java client binding for the GQL Wire Protocol (GWP), implementing the ISO/IEC 39075 wire-level interface over gRPC.

## Requirements

- Java 17+
- Maven 3.8+

## Building

```bash
cd java
mvn clean install
```

The build will:
1. Generate Java classes from the proto files in `../proto/`
2. Compile the binding source code
3. Run the test suite

## Quick Start

```java
import io.grafeodb.gwp.GqlConnection;
import io.grafeodb.gwp.GqlSession;
import io.grafeodb.gwp.ResultCursor;
import io.grafeodb.gwp.Transaction;

import java.util.List;

// Connect and create a session
try (GqlConnection conn = GqlConnection.connect("localhost:50051")) {
    try (GqlSession session = conn.createSession()) {

        // Execute a query
        try (ResultCursor cursor = session.execute("MATCH (n:Person) RETURN n.name")) {
            List<String> columns = cursor.columnNames();

            for (List<Object> row : cursor) {
                System.out.println(row.get(0));
            }
        }

        // Use a transaction
        try (Transaction tx = session.beginTransaction()) {
            tx.execute("INSERT (:Person {name: 'Alice'})").close();
            tx.execute("INSERT (:Person {name: 'Bob'})").close();
            tx.commit();
        }
        // If commit() is not called, the transaction is automatically rolled back
    }
}
```

## API Overview

### Connection

`GqlConnection` manages the gRPC channel to the server.

```java
// Plaintext connection
GqlConnection conn = GqlConnection.connect("localhost:50051");

// TLS connection
GqlConnection conn = GqlConnection.connect("server.example.com:50051", true);
```

### Session

`GqlSession` represents an authenticated session with the server.

```java
GqlSession session = conn.createSession();

// Configure session
session.setGraph("my_graph");
session.setSchema("my_schema");
session.setTimeZone(60); // UTC+1 in minutes

// Reset configuration
session.reset();

// Health check
long timestamp = session.ping();
```

### ResultCursor

`ResultCursor` provides streaming access to query results. It implements `Iterator<List<Object>>` and `AutoCloseable`.

```java
try (ResultCursor cursor = session.execute("MATCH (n) RETURN n.name, n.age")) {
    List<String> columns = cursor.columnNames(); // ["n.name", "n.age"]

    // Option 1: Iterate row by row
    List<Object> row;
    while ((row = cursor.nextRow()) != null) {
        String name = (String) row.get(0);
        Long age = (Long) row.get(1);
    }

    // Option 2: Collect all rows
    List<List<Object>> allRows = cursor.collectRows();

    // Option 3: Use Iterator/for-each
    for (List<Object> r : cursor) {
        // ...
    }

    // Check result status
    ResultCursor.ResultSummary summary = cursor.summary();
    boolean ok = summary.isSuccess();
    long affected = summary.rowsAffected();
}
```

### Transaction

`Transaction` provides explicit transaction control with auto-rollback on close.

```java
try (Transaction tx = session.beginTransaction()) {
    tx.execute("INSERT (:Person {name: $name})", Map.of("name", "Alice")).close();
    tx.commit();
}

// Read-only transaction
try (Transaction tx = session.beginTransaction(true)) {
    try (ResultCursor cursor = tx.execute("MATCH (n) RETURN count(n)")) {
        // ...
    }
    tx.commit();
}
```

### GQL Types

The binding uses Java records for all GQL value types:

| GQL Type         | Java Type        |
|------------------|------------------|
| NULL             | `null`           |
| BOOLEAN          | `Boolean`        |
| INTEGER          | `Long`           |
| FLOAT            | `Double`         |
| STRING           | `String`         |
| BYTES            | `byte[]`         |
| DATE             | `GqlDate`        |
| LOCAL TIME       | `GqlLocalTime`   |
| ZONED TIME       | `GqlZonedTime`   |
| LOCAL DATETIME   | `GqlLocalDateTime` |
| ZONED DATETIME   | `GqlZonedDateTime` |
| DURATION         | `GqlDuration`    |
| LIST             | `List<Object>`   |
| RECORD           | `GqlRecord`      |
| NODE             | `GqlNode`        |
| EDGE             | `GqlEdge`        |
| PATH             | `GqlPath`        |

Temporal types can be converted to `java.time` equivalents:

```java
GqlDate date = (GqlDate) row.get(0);
LocalDate ld = date.toLocalDate();

GqlZonedDateTime zdt = (GqlZonedDateTime) row.get(1);
OffsetDateTime odt = zdt.toOffsetDateTime();
```

### GQLSTATUS

The `GqlStatus` class provides constants for all GQLSTATUS codes and helper methods:

```java
GqlStatus.isSuccess("00000");   // true
GqlStatus.isWarning("01000");   // true
GqlStatus.isNoData("02000");    // true
GqlStatus.isException("42001"); // true
GqlStatus.statusClass("42001"); // "42"
```

### Error Handling

```java
try {
    session.execute("INVALID SYNTAX");
} catch (GqlStatusException e) {
    String code = e.code();          // "42001"
    String msg = e.gqlMessage();     // human-readable message
    boolean isErr = e.isException(); // true
} catch (SessionException e) {
    // Session-level error (expired, not found)
} catch (TransactionException e) {
    // Transaction state error
} catch (GqlException e) {
    // Base GWP error (connection failure, etc.)
}
```

## Project Structure

```
java/
  pom.xml
  src/
    main/java/io/grafeodb/gwp/
      GqlConnection.java          - Connection management
      GqlSession.java             - Session lifecycle and configuration
      ResultCursor.java           - Streaming result cursor
      Transaction.java            - Explicit transaction control
      GqlStatus.java              - GQLSTATUS constants and helpers
      errors/
        GqlException.java         - Base exception
        GqlStatusException.java   - GQL-domain error with status code
        SessionException.java     - Session-level errors
        TransactionException.java - Transaction state errors
      types/
        GqlNode.java              - Property graph node
        GqlEdge.java              - Property graph edge
        GqlPath.java              - Graph path
        GqlRecord.java            - Named field collection
        GqlField.java             - Single record field
        GqlDate.java              - Calendar date
        GqlLocalTime.java         - Time without timezone
        GqlZonedTime.java         - Time with UTC offset
        GqlLocalDateTime.java     - Datetime without timezone
        GqlZonedDateTime.java     - Datetime with UTC offset
        GqlDuration.java          - Temporal duration
      internal/
        ValueConverter.java       - Proto <-> native conversion
    test/java/io/grafeodb/gwp/
      GqlStatusTest.java          - Status helper unit tests
      GqlConnectionTest.java      - Integration tests with mock server
```

## Dependencies

| Dependency | Purpose |
|------------|---------|
| `io.grpc:grpc-netty-shaded` | gRPC transport (Netty-based) |
| `io.grpc:grpc-protobuf` | Protobuf message serialization |
| `io.grpc:grpc-stub` | gRPC stub classes |
| `com.google.protobuf:protobuf-java` | Protobuf runtime |
| `org.junit.jupiter:junit-jupiter` | Testing (test scope) |
| `org.mockito:mockito-core` | Mocking (test scope) |
| `io.grpc:grpc-testing` | In-process gRPC server for tests |
