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
types whose every request names the session, so the server works in that
session's catalog:

```typescript
const catalog = session.catalog();
console.log(catalog.sessionId); // the session's id

const schemas = await catalog.listSchemas();
const graphs = await catalog.listGraphs("default");

catalog.close();
```

A server with authentication refuses catalog requests without a live session
(gRPC `UNAUTHENTICATED`) and answers `NOT_FOUND` for an unknown one, so use
`session.catalog()` there. `conn.createCatalogClient()` gives a client that
names no session (its `sessionId` is `null`), for servers without
authentication.

## Features

- Full TypeScript type safety
- Async iterator support for streaming results
- Transaction support with auto-rollback
- Complete GQL type mapping (nodes, edges, paths, temporals)
- GQLSTATUS error handling

## License

MIT OR Apache-2.0
