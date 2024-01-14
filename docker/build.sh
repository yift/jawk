#!/bin/bash

set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd "$SCRIPT_DIR/.."
cargo build --release --target=x86_64-unknown-linux-musl
rm -rf "$SCRIPT_DIR/target"
mkdir -p "$SCRIPT_DIR/target"
cp "$SCRIPT_DIR/../target/x86_64-unknown-linux-musl/release/jawk" "$SCRIPT_DIR/target"

docker build "$SCRIPT_DIR" -t yift/jawk:0.1