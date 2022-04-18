#!/bin/bash

cargo build --release --example test_classes --target i686-pc-windows-gnu
cargo build --release --example test_classes --target x86_64-pc-windows-gnu
cargo build --release --example test_classes --target i686-unknown-linux-gnu
cargo build --release --example test_classes --target x86_64-unknown-linux-gnu

cargo run --package nekojni_cli