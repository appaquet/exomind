#!/usr/bin/env bash
set -ex
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

REPO_ROOT="$CUR_DIR/.."

OUTPUT=${1:-html}

OUTPUT_ARGS=""
if [[ "$OUTPUT" == "html" ]]; then
	OUTPUT_ARGS="--html"
else
	OUTPUT_ARGS="--lcov --output-path $REPO_ROOT/lcov.info"
fi

rustup component add llvm-tools-preview

cd "$REPO_ROOT"
cargo llvm-cov --workspace \
		--ignore-filename-regex="(protos|exo|clients)/.*" \
		$OUTPUT_ARGS
