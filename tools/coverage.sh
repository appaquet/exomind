#!/usr/bin/env bash
set -ex
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

OUTPUT=${1:-lcov}

OUTPUT_ARGS=""
if [[ "$OUTPUT" == "html" ]]; then
	OUTPUT_ARGS="--html"
else
	OUTPUT_ARGS="--lcov --output-path $CUR_DIR/../lcov.info"
fi

cd "$CUR_DIR/.."
cargo llvm-cov --workspace \
		--exclude=exo \
		--exclude=exocore-client-web \
		--exclude=exocore-client-android \
		--exclude=exocore-client-c \
		--exclude=exocore-apps-macros \
		--exclude=exocore-protos \
		$OUTPUT_ARGS