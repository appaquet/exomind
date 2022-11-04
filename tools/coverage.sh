#!/usr/bin/env bash
set -ex
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

OUTPUT=${1:-html}

OUTPUT_ARGS=""
if [[ "$OUTPUT" == "html" ]]; then
	OUTPUT_ARGS="--html"
else
	OUTPUT_ARGS="--lcov --output-path $CUR_DIR/../lcov.info"
fi

rustup component add llvm-tools-preview

cd "$CUR_DIR/.."
cargo llvm-cov --workspace \
		--ignore-filename-regex="(protos|exo|clients)/.*" \
		$OUTPUT_ARGS
