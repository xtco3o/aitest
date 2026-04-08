#!/usr/bin/env bash
set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
set -x
export CARGO_REGISTRIES_CRATES_IO_PROTOCOL=git
deltmp
cargo update
cargo upgrade --recursive --verbose --incompatible
