#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PROTO_DIR="$(cd "$ROOT/../proto" && pwd)"
OUT_DIR="$ROOT/src/generated"

mkdir -p "$OUT_DIR"

protoc \
  --plugin="protoc-gen-ts_proto=$(npm bin)/protoc-gen-ts_proto" \
  --ts_proto_out="$OUT_DIR" \
  --ts_proto_opt=outputServices=grpc-js \
  --ts_proto_opt=esModuleInterop=true \
  --ts_proto_opt=forceLong=bigint \
  --ts_proto_opt=useDate=false \
  --proto_path="$PROTO_DIR" \
  "$PROTO_DIR/gql_types.proto" \
  "$PROTO_DIR/gql_service.proto"

echo "Generated TypeScript stubs in $OUT_DIR"
