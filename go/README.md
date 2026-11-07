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

## Catalog

`session.Catalog()` returns a catalog client for schemas, graphs and graph
types whose every request names that session:

```go
catalog := session.Catalog()

schemas, err := catalog.ListSchemas(ctx)
if err != nil {
    panic(err)
}
for _, s := range schemas {
    fmt.Println(s.Name, s.GraphCount)
}
```

A request that names an unknown session gets `codes.NotFound`.
`conn.CreateCatalogClient()` gives a client that names no session.
`catalog.SessionID()` returns the session a client names, or `""` when it
names none.

This client does not send credentials in the handshake yet, so it cannot open
a session on a server with authentication (such a server refuses the handshake,
and catalog requests without a session, with `codes.Unauthenticated`). Use the
Rust client against such a server for now.

## Features

- Context-based API following Go conventions
- Streaming result cursor
- Transaction support with defer rollback pattern
- Complete GQL type mapping (nodes, edges, paths, temporals)
- GQLSTATUS error handling

## License

MIT OR Apache-2.0
