#!/bin/bash
# Build script for Materials-Simulato-R

set -e

echo "🦀 Building Materials-Simulato-R..."

# Check Rust version
echo "Rust version:"
rustc --version

# Format check
echo "📝 Checking code formatting..."
cargo fmt --all -- --check

# Clippy
echo "🔍 Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

# Build
echo "🔨 Building workspace..."
cargo build --release --all

echo "✅ Build completed successfully!"
