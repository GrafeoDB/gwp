# gwp-js

TypeScript/JavaScript client for the GQL Wire Protocol (GWP).

## Install

```bash
npm install gwp-js
```

## Quick Start

```typescript
import { GqlConnection } from "gwp-js";

const conn = await GqlConnection.connect("localhost:50051");
const session = await conn.createSession();

const cursor = await session.execute("MATCH (n:Person) RETURN n.name");
for await (const row of cursor) {
  console.log(row);
}

await session.close();
await conn.close();
```

## Catalog

`session.catalog()` returns a catalog client for schemas, graphs and graph
types whose every request names the session; a server that keeps a catalog
per session then uses that session's catalog. The client shares the session's
channel, so `close()` on it leaves the session open:

```typescript
const catalog = session.catalog();
console.log(catalog.sessionId); // the session's id

const schemas = await catalog.listSchemas();
const graphs = await catalog.listGraphs("default");

catalog.close();
```

A named session the server does not know gets gRPC `NOT_FOUND`.
`conn.createCatalogClient()` gives a client that names no session (its
`sessionId` is `null`).

This client does not send credentials in the handshake yet, so it cannot open
a session on a server with authentication (such a server refuses the handshake,
and catalog requests without a session, with `UNAUTHENTICATED`). Use the Rust
client against such a server for now.

## Features

- Full TypeScript type safety
- Async iterator support for streaming results
- Transaction support with auto-rollback
- Complete GQL type mapping (nodes, edges, paths, temporals)
- GQLSTATUS error handling

## License

MIT OR Apache-2.0
