#!/usr/bin/env bash
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd $CUR_DIR/../

# Check if tarpaulin is installed, otherwise we spawn into Docker
# We limit execution to 1 core until https://github.com/xd009642/tarpaulin/issues/190 is fixed
TARPAULIN_VERSION=$(cargo tarpaulin --version)
if [[ $? -ne 0 || $FORCE_DOCKER ]]; then
    echo "Executing through Docker..."
    sudo docker run -it --rm --security-opt seccomp=unconfined -v "$PWD:/volume" xd009642/tarpaulin:develop ./utils/coverage.sh
else
    # First try with all cores, which will fail because of https://github.com/xd009642/tarpaulin/issues/190#issuecomment-491040656
    cargo tarpaulin --exclude-files=3rd --verbose --all --out Html

    # Then execute single core
    taskset -c 0 cargo tarpaulin --verbose --all --out Html \
                        --exclude-files=3rd \
                        --exclude-files="*_capnp.rs" \
                        --exclude-files="**/main.rs" \
                        --exclude=exocore-cli
fi
