#!/bin/bash

# Build the Rust project
echo "Building Rust project..."
cargo build --release

# Run the compiled binary
echo "Starting Roblox Browser Server..."
./target/release/roblox_browser
