#!/bin/bash

./compile_java.sh

cargo build --release --example test_classes
cargo run --package nekojni_cli
