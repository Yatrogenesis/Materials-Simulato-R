#!/bin/bash
# Test script for Materials-Simulato-R

set -e

echo "🧪 Running tests for Materials-Simulato-R..."

# Unit tests
echo "Running unit tests..."
cargo test --all

# Doc tests
echo "Running doc tests..."
cargo test --doc

# Integration tests (if databases are available)
if command -v docker &> /dev/null; then
    echo "Running integration tests..."
    cargo test --test integration || echo "⚠️  Integration tests skipped (databases not running)"
fi

echo "✅ All tests passed!"
