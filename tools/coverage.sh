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

# Use a specific nightly version since they are unstable from time to time
# To check new versions, see: https://rust-lang.github.io/rustup-components-history/
export RUSTUP_TOOLCHAIN=nightly-2021-12-05 
rustup component add llvm-tools-preview

cd "$CUR_DIR/.."
cargo llvm-cov --workspace \
		--exclude=exo \
		--exclude=exocore-client-web \
		--exclude=exocore-client-android \
		--exclude=exocore-client-c \
		--exclude=exocore-apps-macros \
		--exclude=exocore-apps-example \
		--exclude=exocore-protos \
		$OUTPUT_ARGS