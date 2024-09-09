#!/usr/bin/env bash
set -ex
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

export REPO_ROOT="$CUR_DIR/../"
export EXOCORE_ROOT="$REPO_ROOT/exocore"
export EXOMIND_ROOT="$REPO_ROOT/exomind"

cd $REPO_ROOT

echo "Formatting..."
./tools/format.sh

echo "Cargo checking code, tests and benches"
cd $REPO_ROOT
cargo check --workspace --tests --benches --all-features

echo "Running tests..."
cd $REPO_ROOT
cargo test --workspace --all-features

echo "Running clippy..."
$REPO_ROOT/tools/clippy.sh

echo "Pre-checking exocore..."
$EXOCORE_ROOT/tools/pre_commit.sh

echo "Pre-checking exomind..."
$EXOMIND_ROOT/tools/pre_commit.sh