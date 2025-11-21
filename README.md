# Materials-Simulato-R 🦀

**Enterprise Materials Simulation Platform in Rust**

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/status-In%20Development-yellow.svg)](https://github.com/Yatrogenesis/Materials-Simulato-R)

---

## 🎯 Vision

Materials-Simulato-R is the **enterprise Rust refactor** of Materials-SimPro, delivering:

- **100x Performance**: Rust's zero-cost abstractions and memory safety
- **Multi-LLM Integration**: Seamless integration with GPT-4, Claude, Gemini, Mistral, Phi2, TinyLlama
- **Intelligent Fallback**: Automatic degradation to local models
- **Type-Safe Database**: SQLx compile-time checked queries
- **Enterprise-Ready**: Based on AION-R architecture

---

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.75.0+ (MSRV) or 1.82.0+ (recommended)
- **PostgreSQL** 13+
- **MongoDB** 5.0+
- **Redis** 6+
- **Docker** & **Docker Compose** (optional but recommended)

### Installation

```bash
# Clone repository
git clone https://github.com/Yatrogenesis/Materials-Simulato-R.git
cd Materials-Simulato-R

# Check Rust version
rustc --version  # Should be >= 1.75.0

# Build project
cargo build --release

# Run tests
cargo test --all

# Start services (Docker Compose)
docker-compose up -d

# Run API gateway
cargo run --bin api-gateway
```

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────┐
│           MATERIALS-SIMULATO-R PLATFORM                  │
├─────────────────────────────────────────────────────────┤
│  API Layer: Axum + Tower (REST/GraphQL/gRPC/WebSocket) │
├─────────────────────────────────────────────────────────┤
│  Multi-LLM Orchestration: Smart Router + Fallback       │
│  GPT-4 → Claude → Gemini → Mistral → Phi2 → TinyLlama  │
├─────────────────────────────────────────────────────────┤
│  Compute Engine: ML (Candle) + MD + DFT                │
├─────────────────────────────────────────────────────────┤
│  Database: PostgreSQL + MongoDB + Neo4j + Redis         │
├─────────────────────────────────────────────────────────┤
│  Monitoring: Prometheus + Grafana + Tracing             │
└─────────────────────────────────────────────────────────┘
```

### Components from AION-R

✅ **API Gateway** - High-performance request routing
✅ **Authentication** - JWT + Multi-tenant + RBAC
✅ **Database Layer** - SQLx with connection pooling
✅ **Monitoring** - Prometheus metrics + distributed tracing
✅ **Configuration** - Centralized config management

---

## 📚 Project Structure

```
materials-simulato-r/
├── crates/                   # Library crates
│   ├── core/                 # Core types and traits
│   ├── database/             # Database abstraction layer
│   ├── compute/              # Computation engines (ML, MD, DFT)
│   ├── llm/                  # Multi-LLM integration
│   ├── auth/                 # Authentication & authorization
│   ├── api/                  # API layer (REST/GraphQL/gRPC)
│   ├── monitoring/           # Metrics and observability
│   └── cli/                  # CLI interface
│
├── services/                 # Binary services
│   ├── api-gateway/          # Main API gateway
│   ├── compute-worker/       # Compute worker nodes
│   └── llm-orchestrator/     # LLM orchestration service
│
├── deployment/               # Deployment configs
│   ├── docker/               # Docker & Compose
│   ├── kubernetes/           # K8s manifests
│   └── terraform/            # Infrastructure as Code
│
├── tests/                    # Integration tests
├── benches/                  # Performance benchmarks
├── docs/                     # Documentation
└── scripts/                  # Utility scripts
```

---

## 🔧 Technology Stack

### Core
- **Rust** 1.75.0+ (MSRV) / 1.82.0+ (stable target)
- **Tokio** - Async runtime
- **Axum** - Web framework
- **SQLx** - Database (compile-time checked)

### Scientific Computing
- **ndarray** - N-dimensional arrays (NumPy replacement)
- **nalgebra** - Linear algebra
- **rayon** - Data parallelism
- **polars** - DataFrames (pandas replacement)
- **candle** - ML in pure Rust
- **tch-rs** - PyTorch bindings

### LLM Integration
- **async-openai** - OpenAI GPT-4, GPT-3.5
- **anthropic-sdk-rust** - Claude-3.5, Claude-3
- **google-generativeai** - Gemini Pro
- **mistralrs** - Mistral, Mixtral (local/API)
- **llm** - Local models (Phi2, TinyLlama)
- **candle** - Local inference (GGUF)

### Databases
- **PostgreSQL** (SQLx)
- **MongoDB** (async driver)
- **Neo4j** (neo4rs)
- **Redis** (redis-rs)

---

## 🧪 Development

### Build

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Check without building
cargo check --all
```

### Test

```bash
# Run all tests
cargo test --all

# Run specific test
cargo test test_material_creation

# Run with output
cargo test -- --nocapture

# Run integration tests
cargo test --test integration
```

### Benchmarks

```bash
# Run benchmarks
cargo bench

# Specific benchmark
cargo bench --bench compute_benchmarks
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Security audit
cargo audit

# Check dependencies
cargo outdated
```

---

## 🌐 Multi-LLM Integration

### Supported Providers

| Provider | Models | Cost | Speed | Offline |
|----------|--------|------|-------|---------|
| **OpenAI** | GPT-4, GPT-3.5 | $$$ | Medium | ❌ |
| **Anthropic** | Claude-3.5, Claude-3 | $$ | Fast | ❌ |
| **Google** | Gemini Pro | $$ | Fast | ❌ |
| **Mistral** | Mixtral-8x7B, Mistral-7B | $ | Medium | ✅ |
| **Local** | Phi2, TinyLlama | Free | Fast | ✅ |

### Fallback Chain

```
GPT-4 → Claude-3.5 → Gemini → Mixtral-8x7B → Mistral-7B → Phi2 → TinyLlama
(Cloud)  (Cloud)     (Cloud)   (Local)        (Local)     (Local) (Local)
```

**Features:**
- ✅ Automatic failover (< 100ms)
- ✅ Circuit breaker pattern
- ✅ Cost optimization
- ✅ 100% offline availability
- ✅ Smart routing (cost/speed/quality)

---

## 📊 Performance Targets

| Metric | Python | Rust Target | Improvement |
|--------|--------|-------------|-------------|
| **API Latency (p95)** | 200ms | 20ms | 10x |
| **Energy calc (ML, 1K atoms)** | 100ms | 10ms | 10x |
| **Energy calc (ML, 10K atoms)** | 1s | 100ms | 10x |
| **Memory usage** | 500MB | 50MB | 10x |
| **Startup time** | 5s | 100ms | 50x |

---


## 📖 Materials-SimPro Data Integration

### 🚧 Por Desarrollar - Data Consumption from Public Documentation

Materials-Simulato-R is designed to consume scientific data and documentation from the **[materials-simpro-releases](https://github.com/Yatrogenesis/materials-simpro-releases)** public repository.

#### Planned Integration Features

**Data Sources** (from materials-simpro-releases):
- 📐 **FEM Solver Documentation**: Element libraries, mesh generation algorithms
- 🧬 **Molecular Dynamics Data**: Force field parameters, interatomic potentials
- 🤖 **ML Model Architectures**: Pre-trained models for property prediction
- 📊 **Materials Database**: Property values, experimental datasets
- ⚙️ **Optimization Algorithms**: Multi-objective optimization strategies

**Current Status**:
- ✅ Architecture documented in materials-simpro-releases
- ✅ Data schema defined
- 🚧 Rust integration layer in development
- 🚧 Automatic sync mechanism planned

**Related Repository**:
- **Documentation**: [materials-simpro-releases](https://github.com/Yatrogenesis/materials-simpro-releases)
- **Python Version**: Materials-SimPro (enterprise license)
- **Data Format**: JSON, HDF5, ONNX, SQLite

---

## 🗺️ Roadmap

### Phase 0: Setup (Weeks 1-2) ✅
- [x] Repository created
- [x] TDD completed
- [ ] Workspace configured
- [ ] CI/CD pipeline

### Phase 1: Core Infrastructure (Weeks 3-8) 🚧
- [ ] Database layer (PostgreSQL, MongoDB, Neo4j, Redis)
- [ ] Core types and traits
- [ ] ML engine basic (Candle)

### Phase 2: Multi-LLM (Weeks 9-14) 🔜
- [ ] LLM provider abstraction
- [ ] Smart router & fallback
- [ ] Local model integration

### Phase 3: Compute Engine (Weeks 15-22)
- [ ] Molecular dynamics
- [ ] Property calculators
- [ ] DFT bridges

### Phase 4: API & Services (Weeks 23-28)
- [ ] REST API (Axum)
- [ ] GraphQL API
- [ ] gRPC services

### Phase 5: Production (Weeks 29-32)
- [ ] Monitoring & observability
- [ ] Kubernetes deployment
- [ ] CI/CD automation

---

## 🔐 Security

- **Authentication**: JWT-based with multi-tenant support
- **Authorization**: RBAC (Role-Based Access Control)
- **Encryption**: TLS 1.3, AES-256 at rest
- **Audit Logging**: Complete audit trail
- **Circuit Breaker**: Fault tolerance
- **Rate Limiting**: Token bucket algorithm

---

## 📖 Documentation

- **[TDD](TDD_Materials-Simulato-R.md)** - Complete technical design document
- **[API Reference](docs/api/)** - API documentation
- **[Architecture](docs/architecture/)** - System architecture
- **[Deployment](docs/deployment/)** - Deployment guides

---

## 🤝 Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Run tests and linters
4. Submit a pull request

**Standards:**
- Rust 1.75.0+ (MSRV)
- `cargo fmt` for formatting
- `cargo clippy` with no warnings
- Tests for all new features
- Documentation for public APIs

---

## 📜 License

This project is licensed under the **MIT License** - see [LICENSE](LICENSE) for details.

---

## 📞 Contact

- **GitHub**: [Yatrogenesis/Materials-Simulato-R](https://github.com/Yatrogenesis/Materials-Simulato-R)
- **Email**: info@yatrogenesis.com
- **Issues**: [GitHub Issues](https://github.com/Yatrogenesis/Materials-Simulato-R/issues)

---

## 🏆 Acknowledgments

Based on:
- **Materials-SimPro** - Original Python platform
- **AION-R** - Enterprise Rust architecture
- **Rust Community** - Amazing ecosystem

---

**Status**: 🟢 Active Development
**Version**: 1.0.0
**MSRV**: 1.75.0
**Last Updated**: 2025-11-07

🦀 **Building the future of materials science with Rust!** 🚀
