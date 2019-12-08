#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$CUR_DIR"

cbindgen --config cbindgen.toml --crate exocore-client-ios --output exocore.h
