# gwp-py

Python client for the GQL Wire Protocol (GWP).

## Install

```bash
pip install gwp-py
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

## Features

- Async-first API built on `grpcio.aio`
- Full GQL type support (nodes, edges, paths, temporals, lists, maps)
- Transaction support with auto-rollback context managers
- GQLSTATUS error handling

## License

MIT OR Apache-2.0
