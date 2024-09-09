#!/usr/bin/env bash
set -ex
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

./exocore/tools/generate_protos.sh
./exomind/tools/generate_protos.sh
