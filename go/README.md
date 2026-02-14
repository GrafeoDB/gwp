# gwp-go

Go client for the GQL Wire Protocol (GWP).

## Install

```bash
go get github.com/GrafeoDB/gql-wire-protocol/go
```

## Quick Start

```go
package main

import (
    "context"
    "fmt"

    gwp "github.com/GrafeoDB/gql-wire-protocol/go"
)

func main() {
    ctx := context.Background()

    conn, err := gwp.Connect(ctx, "localhost:50051")
    if err != nil {
        panic(err)
    }
    defer conn.Close()

    session, err := conn.CreateSession(ctx)
    if err != nil {
        panic(err)
    }
    defer session.Close(ctx)

    cursor, err := session.Execute(ctx, "MATCH (n:Person) RETURN n.name", nil)
    if err != nil {
        panic(err)
    }

    for {
        row, err := cursor.NextRow()
        if err != nil {
            panic(err)
        }
        if row == nil {
            break
        }
        fmt.Println(row)
    }
}
```

## Features

- Context-based API following Go conventions
- Streaming result cursor
- Transaction support with defer rollback pattern
- Complete GQL type mapping (nodes, edges, paths, temporals)
- GQLSTATUS error handling

## License

MIT OR Apache-2.0
