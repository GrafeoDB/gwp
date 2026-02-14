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

## Features

- Full TypeScript type safety
- Async iterator support for streaming results
- Transaction support with auto-rollback
- Complete GQL type mapping (nodes, edges, paths, temporals)
- GQLSTATUS error handling

## License

MIT OR Apache-2.0
