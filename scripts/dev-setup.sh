#!/bin/bash
# Development environment setup script

set -e

echo "🚀 Setting up Materials-Simulato-R development environment..."

# Check prerequisites
echo "Checking prerequisites..."
command -v rustc >/dev/null 2>&1 || { echo "❌ Rust not installed. Install from https://rustup.rs/"; exit 1; }
command -v docker >/dev/null 2>&1 || { echo "⚠️  Docker not found. Some features will be limited."; }

# Create .env file if not exists
if [ ! -f .env ]; then
    echo "📝 Creating .env from .env.example..."
    cp .env.example .env
    echo "⚠️  Please edit .env and add your API keys!"
fi

# Install Rust components
echo "📦 Installing Rust components..."
rustup component add rustfmt clippy rust-src rust-analyzer

# Build project
echo "🔨 Building project..."
cargo build

# Start databases (if Docker available)
if command -v docker &> /dev/null; then
    echo "🐳 Starting databases with Docker Compose..."
    docker-compose up -d postgres mongodb redis neo4j

    echo "⏳ Waiting for databases to be ready..."
    sleep 10

    echo "✅ Databases started!"
else
    echo "⚠️  Docker not available. Skipping database setup."
fi

echo "✅ Development environment setup complete!"
echo ""
echo "Next steps:"
echo "  1. Edit .env and add your API keys"
echo "  2. Run 'cargo run --bin api-gateway' to start the API server"
echo "  3. Run 'cargo test' to run tests"
