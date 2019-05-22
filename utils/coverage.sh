#!/usr/bin/env bash
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd $CUR_DIR/../

OUTPUT=${1:-Html}

# Check if tarpaulin is installed, otherwise we spawn into Docker
# We limit execution to 1 core until https://github.com/xd009642/tarpaulin/issues/190 is fixed
TARPAULIN_VERSION=$(cargo tarpaulin --version)
if [[ $? -ne 0 || $FORCE_DOCKER ]]; then
    echo "Executing through Docker..."
    sudo docker run --rm --security-opt seccomp=unconfined -v "$PWD:/volume" appaquet/tarpaulin:0.7.0 ./utils/coverage.sh $OUTPUT
else
    # First try with all cores for faster compilation, which will fail because of https://github.com/xd009642/tarpaulin/issues/190#issuecomment-491040656
    cargo tarpaulin --verbose --all --out $OUTPUT \
                        --exclude="exocore-cli" \
                        --exclude="exocore-client-wasm" \
                        --exclude="exocore-client-android" \
                        --exclude-files=3rd \
                        --exclude-files="*_capnp.rs" \
                        --exclude-files="**/main.rs" \
                        --exclude-files="cli/**" \
                        --exclude-files="clients/**"

    # Then execute single core
    taskset -c 0 cargo tarpaulin --verbose --all --out $OUTPUT \
                        --exclude="exocore-cli" \
                        --exclude="exocore-client-wasm" \
                        --exclude="exocore-client-android" \
                        --exclude-files=3rd \
                        --exclude-files="*_capnp.rs" \
                        --exclude-files="**/main.rs" \
                        --exclude-files="cli/**" \
                        --exclude-files="clients/**"
fi
