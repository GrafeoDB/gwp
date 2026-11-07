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
    async with conn.create_session() as session:
        cursor = await session.execute("MATCH (n:Person) RETURN n.name")
        async for row in cursor:
            print(row)


asyncio.run(main())
```

## Catalog

`session.catalog()` returns a `CatalogClient` that names the session on every
request, so the server works in that session's catalog:

```python
async with conn.create_session() as session:
    catalog = session.catalog()
    for schema in await catalog.list_schemas():
        print(schema.name, schema.graph_count)
    graphs = await catalog.list_graphs("default")
```

A server with authentication needs this session-bound client: it refuses a
catalog request that names no session (`UNAUTHENTICATED`) and one that names an
unknown or closed session (`NOT_FOUND`). `conn.create_catalog_client()` gives a
client that names no session, for servers without authentication.
`catalog.session_id` is the session a client names, or `None` for none.

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
