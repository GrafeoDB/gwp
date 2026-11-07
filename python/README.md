# gwp-py

Python client for the GQL Wire Protocol (GWP).

## Install

```bash
uv add gwp-py
```

## Quick Start

```python
import asyncio
from gwp_py import GqlConnection


async def main():
    conn = await GqlConnection.connect("localhost:50051")
    async with await conn.create_session() as session:
        cursor = await session.execute("MATCH (n:Person) RETURN n.name")
        async for row in cursor:
            print(row)


asyncio.run(main())
```

## Catalog

`session.catalog()` returns a `CatalogClient` that names the session on every
request; a server that keeps a catalog per session then uses that session's
catalog:

```python
async def main():
    conn = await GqlConnection.connect("localhost:50051")
    async with await conn.create_session() as session:
        catalog = session.catalog()
        for schema in await catalog.list_schemas():
            print(schema.name, schema.graph_count)
        graphs = await catalog.list_graphs("default")
    await conn.close()


asyncio.run(main())
```

A named session that is unknown or closed gets `NOT_FOUND`.
`conn.create_catalog_client()` gives a client that names no session.
`catalog.session_id` is the session a client names, or `None` for none.

This client does not send credentials in the handshake yet, so it cannot open
a session on a server with authentication (such a server refuses the handshake,
and catalog requests without a session, with `UNAUTHENTICATED`). Use the Rust
client against such a server for now.

## Features

- Async-first API built on `grpcio.aio`
- Full GQL type support (nodes, edges, paths, temporals, lists, maps)
- Transaction support with auto-rollback context managers
- GQLSTATUS error handling
- Typed write counters: `await cursor.counters()` returns a `Counters` with
  `nodes_created`, `edges_created`, `properties_set` and so on, plus
  `contains_updates()`; the raw map stays on `summary.counters`

## License

MIT OR Apache-2.0
