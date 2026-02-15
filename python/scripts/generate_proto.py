#!/usr/bin/env python3
"""Generate Python gRPC stubs from proto files."""

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).parent.parent
PROTO_DIR = ROOT.parent / "proto"
OUT_DIR = ROOT / "src" / "gwp_py" / "_generated"


def main():
    OUT_DIR.mkdir(parents=True, exist_ok=True)

    proto_files = [
        str(PROTO_DIR / "gql_types.proto"),
        str(PROTO_DIR / "gql_service.proto"),
    ]

    cmd = [
        sys.executable,
        "-m",
        "grpc_tools.protoc",
        f"--proto_path={PROTO_DIR}",
        f"--python_out={OUT_DIR}",
        f"--pyi_out={OUT_DIR}",
        f"--grpc_python_out={OUT_DIR}",
        *proto_files,
    ]

    print(f"Generating stubs in {OUT_DIR}")
    subprocess.check_call(cmd)

    # Fix absolute imports to be package-relative.
    # grpc_tools.protoc generates `import gql_types_pb2` but we need
    # `from . import gql_types_pb2` since the files live inside a package.
    import re

    for py_file in OUT_DIR.glob("*.py"):
        text = py_file.read_text()
        fixed = re.sub(
            r"^(import gql_\w+ as )",
            r"from . \1",
            text,
            flags=re.MULTILINE,
        )
        if fixed != text:
            py_file.write_text(fixed)
            print(f"  Fixed imports in {py_file.name}")

    init_file = OUT_DIR / "__init__.py"
    if not init_file.exists():
        init_file.write_text("")

    print("Done.")


if __name__ == "__main__":
    main()
