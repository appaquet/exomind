#!/usr/bin/env bash
set -ex
CUR_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

COVERAGE_DIR="$CUR_DIR/../coverage"
OUTPUT=${1:-Html}

##
## See https://github.com/mozilla/grcov#grcov-with-travis
##

if [[ -d $CUR_DIR/../target ]]; then
  find $CUR_DIR/../target -name "*.gc*" -delete
fi

export RUSTUP_TOOLCHAIN=nightly-2021-07-06 # if changed, change in push-tester.yml. Use https://rust-lang.github.io/rustup-components-history/ to get a nightly with all components

export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort"
export CARGO_OPTIONS="--all --all-features --exclude=exo --exclude=exocore-client-web --exclude=exocore-client-android --exclude=exocore-client-c --exclude=exocore-apps-macros"

cd $CUR_DIR/../
cargo clean -p exocore-protos
cargo test $CARGO_OPTIONS

mkdir -p $COVERAGE_DIR
zip -0 $COVERAGE_DIR/ccov.zip `find . \( -name "*exocore*.gc*" \) -print`;

grcov $COVERAGE_DIR/ccov.zip -s . -t lcov --llvm -o $COVERAGE_DIR/lcov.info \
	--ignore-not-existing \
	--ignore "clients/*" \
	--ignore "exo/*" \
	--ignore "/*" \
	--ignore "protos/src/generated/*"

if [[ "$OUTPUT" == "Html" ]]; then
  # genhtml is provided by `lcov` package
	genhtml -o $COVERAGE_DIR/ --show-details --highlight --ignore-errors source --legend $COVERAGE_DIR/lcov.info
else
	bash <(curl -s https://codecov.io/bash) -f $COVERAGE_DIR/lcov.info;
fi
