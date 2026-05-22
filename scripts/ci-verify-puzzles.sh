#!/usr/bin/env bash
# Build the base image, then run `bashlings verify` against every puzzle.
# Invoked by .github/workflows/ci.yml.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "==> Building base image"
podman build -t bashlings-base:latest containers/

echo "==> Building runner (release)"
cargo build --release

echo "==> Running bashlings verify"
./target/release/bashlings -v verify
