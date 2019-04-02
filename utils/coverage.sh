#!/usr/bin/env bash
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd $CUR_DIR/../

# Check if tarpaulin is installed, otherwise we spawn into Docker
# We limit execution to 1 core until https://github.com/xd009642/tarpaulin/issues/190 is fixed
TARPAULIN_VERSION=$(cargo tarpaulin --version)
if [[ $? -ne 0 || $FORCE_DOCKER ]]; then
    sudo docker run -it --rm --security-opt seccomp=unconfined -v "$PWD:/volume" xd009642/tarpaulin:develop ./utils/coverage.sh
else
    taskset -c 0 cargo tarpaulin --exclude-files=3rd --verbose --exclude-files="*_capnp.rs" --all --out Html
fi
