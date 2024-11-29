#!/usr/bin/env bash
set -e
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

EXOCORE_WEB_EXAMPLE="$CUR_DIR/../"
EXOCORE_ROOT="$EXOCORE_WEB_EXAMPLE/../../"

cd $EXOCORE_WEB_EXAMPLE
yarn exec playwright test
