# Materials-Simulato-R - Documento Técnico de Desarrollo (TDD)
## Enterprise Materials Simulation Platform in Rust

**Versión:** 1.0.0
**Fecha:** 2025-11-07
**Estado:** 🟢 Fase de Diseño
**Clasificación:** Enterprise Research & Development
**Rust MSRV:** 1.75.0 (Minimum Supported Rust Version)
**Rust Stable Target:** 1.82.0+

---

## 📋 RESUMEN EJECUTIVO

**Materials-Simulato-R** es la refactorización enterprise de Materials-SimPro de Python a Rust, enfocada en:

### Objetivos Principales

🎯 **Performance Crítico**: 100x más rápido que la versión Python
🎯 **Multi-LLM Agnóstico**: Integración transparente con GPT-4, Claude, Gemini, Mistral, Phi2, TinyLlama, Mixtral
🎯 **Fallback Inteligente**: Degradación automática a modelos pequeños y eficientes
🎯 **Base de Datos Rust**: Migración completa de PostgreSQL/MongoDB con SQLx y tipos seguros
🎯 **Procesos Centrales al 100%**: Enfoque en core functionality antes de cloud/seguridad
🎯 **Enterprise-Ready**: Arquitectura basada en AION-R con seguridad y escalabilidad

### Componentes Aprovechados de AION-R

✅ **API Gateway** - Axum + Tower para alta performance
✅ **Authentication** - JWT + Multi-tenant + RBAC
✅ **Database Layer** - SQLx + Connection pooling
✅ **Monitoring** - Prometheus + Grafana
✅ **Configuración** - Sistema centralizado de config
✅ **Error Handling** - thiserror + anyhow patterns

---

## 🔧 MATRIZ DE COMPATIBILIDAD RUST Y LIBRERÍAS

### Versiones de Rust

| MSRV (Minimum) | Stable (Target) | Testing | Notas |
|----------------|----------------|---------|-------|
| **1.75.0** | **1.82.0** | 1.83.0-beta | MSRV para compatibilidad offline (Set A) |
| 2021 Edition | 2021 Edition | 2024 Edition (futuro) | Edition estándar |

**Política de Compatibilidad:**
- **MSRV 1.75.0**: Garantiza compatibilidad con sistemas legacy
- **Stable 1.82.0+**: Versión recomendada para desarrollo
- **Testing Beta**: Preparación para futuras versiones

### Core Dependencies Matrix

| Categoría | Crate | Versión | MSRV Compatible | Notas |
|-----------|-------|---------|-----------------|-------|
| **Async Runtime** | tokio | 1.35 | ✅ 1.70+ | Full features, multi-threaded |
| **Web Framework** | axum | 0.7 | ✅ 1.75+ | Type-safe, high-performance |
| **HTTP Client** | reqwest | 0.11 | ✅ 1.70+ | Async HTTP con rustls |
| **Serialization** | serde | 1.0 | ✅ 1.31+ | JSON, YAML, TOML support |
| **Database (SQL)** | sqlx | 0.7 | ✅ 1.75+ | Async, compile-time checked |
| **Database (NoSQL)** | mongodb | 2.8 | ✅ 1.70+ | Async MongoDB driver |
| **Caching** | redis | 0.24 | ✅ 1.70+ | Redis con tokio |
| **Graph DB** | neo4rs | 0.7 | ✅ 1.70+ | Neo4j async driver |
| **Crypto** | argon2 | 0.5 | ✅ 1.65+ | Password hashing |
| **JWT** | jsonwebtoken | 9.2 | ✅ 1.65+ | Token auth |
| **Logging** | tracing | 0.1 | ✅ 1.63+ | Structured logging |
| **Metrics** | metrics | 0.22 | ✅ 1.70+ | Prometheus compatible |
| **Config** | config | 0.14 | ✅ 1.70+ | Layered configuration |
| **CLI** | clap | 4.4 | ✅ 1.74+ | Derive-based CLI |
| **Error** | thiserror | 1.0 | ✅ 1.56+ | Error derive macros |
| **Error** | anyhow | 1.0 | ✅ 1.39+ | Flexible error handling |

### Scientific Computing Dependencies

| Crate | Versión | MSRV | Propósito | Reemplazo de |
|-------|---------|------|-----------|--------------|
| **ndarray** | 0.15 | 1.64+ | Arrays N-dimensionales | NumPy |
| **nalgebra** | 0.32 | 1.65+ | Linear algebra | NumPy/SciPy |
| **rayon** | 1.8 | 1.63+ | Data parallelism | multiprocessing |
| **polars** | 0.36 | 1.71+ | DataFrames | pandas |
| **linfa** | 0.7 | 1.70+ | Machine learning | scikit-learn |
| **tch-rs** | 0.15 | 1.70+ | PyTorch bindings | PyTorch |
| **candle** | 0.3 | 1.75+ | ML en Rust puro | Custom ML |
| **faer** | 0.16 | 1.75+ | Linear algebra (alt) | NumPy/LAPACK |

### LLM Integration Dependencies

| Crate/SDK | Versión | API Support | Modelos | Fallback |
|-----------|---------|-------------|---------|----------|
| **async-openai** | 0.18 | OpenAI | GPT-4, GPT-3.5 | ✅ |
| **anthropic-sdk-rust** | 0.1 | Anthropic | Claude-3.5, Claude-3 | ✅ |
| **google-generativeai** | Custom | Google | Gemini Pro, Ultra | ✅ |
| **mistralrs** | 0.1 | Local/API | Mistral-7B, Mixtral-8x7B | ✅ Primary |
| **llm** | 0.2 | Local | Phi2, TinyLlama, Llama2 | ✅ Local fallback |
| **candle** | 0.3 | Local Inference | GGUF models | ✅ Offline mode |
| **tokenizers** | 0.15 | Tokenization | Universal | ✅ |

### Database Drivers Compatibility

| Database | Driver Crate | Versión | Async | Connection Pool | Migrations |
|----------|-------------|---------|-------|-----------------|------------|
| **PostgreSQL** | sqlx | 0.7 | ✅ | ✅ | ✅ |
| **MongoDB** | mongodb | 2.8 | ✅ | ✅ | Manual |
| **Neo4j** | neo4rs | 0.7 | ✅ | ✅ | Manual |
| **Redis** | redis | 0.24 | ✅ | ✅ | N/A |
| **Elasticsearch** | elasticsearch | 8.12 | ✅ | ✅ | N/A |

### Platform Compatibility

| Platform | Rust Support | Build Status | Deployment |
|----------|--------------|--------------|------------|
| **Linux** (x86_64) | ✅ Tier 1 | ✅ Primary | Docker, K8s |
| **Linux** (aarch64) | ✅ Tier 1 | ✅ Tested | ARM servers |
| **macOS** (x86_64) | ✅ Tier 1 | ✅ Dev | Local |
| **macOS** (aarch64/M1+) | ✅ Tier 1 | ✅ Dev | Local |
| **Windows** (x86_64) | ✅ Tier 1 | ⚠️ Limited | Local/WSL2 |
| **Windows** (MSVC) | ✅ Tier 1 | ✅ Native | Native builds |

---

## 🏗️ ARQUITECTURA DEL SISTEMA

### Arquitectura General

```
┌─────────────────────────────────────────────────────────────────────┐
│                  MATERIALS-SIMULATO-R PLATFORM                       │
│                    (Rust Enterprise Edition)                         │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│                         API LAYER (Axum)                             │
├─────────────────────────────────────────────────────────────────────┤
│  REST API  │  GraphQL  │  gRPC  │  WebSocket  │  CLI Interface      │
│  (Public)  │ (Complex) │ (Fast) │ (Real-time) │  (Local/Remote)     │
└─────────────────────────────────────────────────────────────────────┘
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    MULTI-LLM ORCHESTRATION LAYER                     │
├─────────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │
│  │ Cloud LLMs   │  │ Local Models │  │ Fallback     │              │
│  ├──────────────┤  ├──────────────┤  ├──────────────┤              │
│  │ GPT-4        │  │ Mistral-7B   │  │ TinyLlama    │              │
│  │ Claude-3.5   │  │ Mixtral-8x7B │  │ Phi2         │              │
│  │ Gemini Pro   │  │ Llama2-13B   │  │ SmolLM       │              │
│  └──────────────┘  └──────────────┘  └──────────────┘              │
│                                                                       │
│  🔄 Smart Router: Cost → Speed → Availability → Offline             │
│  🛡️ Circuit Breaker: Auto-failover en < 100ms                       │
│  📊 Load Balancer: Rate limiting + quota management                 │
└─────────────────────────────────────────────────────────────────────┘
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│              CORE SIMULATION ENGINE (Multi-Fidelity)                 │
├─────────────────────────────────────────────────────────────────────┤
│  Level 1: ML Methods (Candle + tch-rs)                              │
│  ├─ Neural Network Potentials (NNP)                                 │
│  ├─ Graph Neural Networks (GNN)                                     │
│  └─ Equivariant Models (E3NN-rs)                                    │
│                                                                       │
│  Level 2: Classical MD (Custom Rust Engine)                         │
│  ├─ LAMMPS FFI bindings                                             │
│  ├─ Custom parallel MD engine                                       │
│  └─ GPU-accelerated kernels                                         │
│                                                                       │
│  Level 3: DFT Interface (FFI to C/C++)                              │
│  ├─ VASP connector (via Python/C bridge)                            │
│  ├─ Quantum ESPRESSO connector                                      │
│  └─ Native lightweight DFT (future)                                 │
└─────────────────────────────────────────────────────────────────────┘
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    DATABASE LAYER (Type-Safe)                        │
├─────────────────────────────────────────────────────────────────────┤
│  PostgreSQL (SQLx)  │  MongoDB  │  Neo4j  │  Redis  │  S3/MinIO    │
│  ├─ Materials       │  ├─ Props │ ├─ Graph│ ├─Cache │ ├─ Files     │
│  ├─ Calculations    │  ├─ Docs  │ ├─ Sim  │ ├─Queue │ ├─ Models    │
│  ├─ Users/Tenants   │  └─ Meta  │ └─ Net  │ └─Sess  │ └─ Results   │
│  └─ Audit logs      │           │         │         │              │
└─────────────────────────────────────────────────────────────────────┘
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                   MONITORING & OBSERVABILITY                         │
├─────────────────────────────────────────────────────────────────────┤
│  Prometheus  │  Grafana  │  Tracing  │  Jaeger  │  ELK Stack       │
└─────────────────────────────────────────────────────────────────────┘
```

### Workspace Structure (Cargo Workspace)

```
materials-simulato-r/
├── Cargo.toml                          # Workspace root
├── Cargo.lock
├── rust-toolchain.toml                 # MSRV specification
├── .cargo/config.toml                  # Build configuration
│
├── crates/                             # Library crates
│   ├── core/                           # Core types and traits
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── material.rs             # Material structure types
│   │       ├── property.rs             # Property types
│   │       ├── error.rs                # Error types
│   │       └── config.rs               # Configuration
│   │
│   ├── database/                       # Database abstraction
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── postgres.rs             # PostgreSQL with SQLx
│   │       ├── mongo.rs                # MongoDB driver
│   │       ├── neo4j.rs                # Neo4j driver
│   │       ├── redis.rs                # Redis cache
│   │       └── migrations/             # SQL migrations
│   │
│   ├── compute/                        # Computation engines
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── ml_engine.rs            # ML methods (Candle)
│   │       ├── md_engine.rs            # Molecular dynamics
│   │       ├── dft_bridge.rs           # DFT connectors
│   │       └── multi_fidelity.rs       # Adaptive method selection
│   │
│   ├── llm/                            # LLM integration
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── router.rs               # Smart LLM routing
│   │       ├── providers/
│   │       │   ├── openai.rs           # GPT-4, GPT-3.5
│   │       │   ├── anthropic.rs        # Claude
│   │       │   ├── google.rs           # Gemini
│   │       │   ├── mistral.rs          # Mistral API
│   │       │   └── local.rs            # Local models
│   │       ├── fallback.rs             # Fallback logic
│   │       └── circuit_breaker.rs      # Fault tolerance
│   │
│   ├── auth/                           # Authentication (from AION-R)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── jwt.rs                  # JWT handling
│   │       ├── rbac.rs                 # Role-based access
│   │       └── multi_tenant.rs         # Multi-tenancy
│   │
│   ├── api/                            # API layer
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── rest.rs                 # REST API (Axum)
│   │       ├── graphql.rs              # GraphQL (async-graphql)
│   │       ├── grpc.rs                 # gRPC (tonic)
│   │       └── websocket.rs            # WebSocket streams
│   │
│   ├── monitoring/                     # Monitoring (from AION-R)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── metrics.rs              # Prometheus metrics
│   │       ├── tracing.rs              # Distributed tracing
│   │       └── health.rs               # Health checks
│   │
│   └── cli/                            # CLI interface
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── commands/
│           │   ├── simulate.rs         # Run simulations
│           │   ├── query.rs            # Database queries
│           │   └── discover.rs         # AI discovery
│           └── output.rs               # Output formatting
│
├── services/                           # Binary services
│   ├── api-gateway/                    # Main API gateway
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   │
│   ├── compute-worker/                 # Compute worker nodes
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   │
│   └── llm-orchestrator/               # LLM orchestration service
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
│
├── tests/                              # Integration tests
│   ├── integration/
│   ├── performance/
│   └── compatibility/
│
├── benches/                            # Benchmarks
│   ├── compute_benchmarks.rs
│   └── llm_benchmarks.rs
│
├── docs/                               # Documentation
│   ├── architecture/
│   ├── api/
│   └── deployment/
│
├── deployment/                         # Deployment configs
│   ├── docker/
│   │   ├── Dockerfile.api
│   │   ├── Dockerfile.worker
│   │   └── docker-compose.yml
│   │
│   ├── kubernetes/
│   │   ├── namespace.yaml
│   │   ├── deployments/
│   │   ├── services/
│   │   └── configmaps/
│   │
│   └── terraform/
│       ├── main.tf
│       ├── variables.tf
│       └── modules/
│
└── scripts/                            # Utility scripts
    ├── build.sh
    ├── test.sh
    ├── deploy.sh
    └── benchmark.sh
```

---

## 🚀 ROADMAP DE IMPLEMENTACIÓN

### Fase 0: Preparación y Setup (Semanas 1-2)

**Objetivos:**
- Configurar infraestructura de desarrollo
- Establecer CI/CD
- Definir estándares de código

**Tareas:**
- [x] Crear repositorio GitHub
- [ ] Configurar Cargo workspace
- [ ] Setup CI/CD (GitHub Actions)
- [ ] Configurar pre-commit hooks
- [ ] Establecer coding standards (rustfmt, clippy)
- [ ] Setup Docker development environment
- [ ] Configurar documentación (mdBook)

**Entregables:**
- Workspace Rust funcional
- CI/CD pipeline operativo
- Guías de contribución

---

### Fase 1: Core Infrastructure (Semanas 3-8) - CRÍTICO

**Objetivo:** Procesos centrales al 100% operativos

#### Milestone 1.1: Database Layer (Semanas 3-4)

**Tareas:**
```rust
// PostgreSQL with SQLx
- [ ] Definir schema materials (structures, properties, metadata)
- [ ] Implementar migrations con sqlx-cli
- [ ] Connection pooling (deadpool-postgres)
- [ ] CRUD operations con type safety
- [ ] Query builder patterns
- [ ] Transaction management

// MongoDB
- [ ] Schema para properties flexibles (JSONB equivalente)
- [ ] Async driver setup
- [ ] Indexing strategies

// Neo4j
- [ ] Graph schema para similarity networks
- [ ] Neo4rs driver integration
- [ ] Cypher query builders

// Redis
- [ ] Caching layer setup
- [ ] Session storage
- [ ] Job queue (redis-backed)
```

**Criterios de Aceptación:**
- ✅ 1000 materials insertados en < 1s
- ✅ Queries complejas < 10ms (cached)
- ✅ Type-safe en compile time
- ✅ Connection pool efficiency > 95%

#### Milestone 1.2: Core Types & Traits (Semanas 4-5)

```rust
// Core material types
- [ ] Structure types (lattice, sites, symmetry)
- [ ] Property types (formation_energy, band_gap, etc.)
- [ ] Error types (thiserror-based)
- [ ] Configuration system
- [ ] Serialization/Deserialization (serde)

// Trait system
- [ ] ComputationMethod trait
- [ ] PropertyCalculator trait
- [ ] DatabaseBackend trait
- [ ] LLMProvider trait
```

**Entregables:**
```rust
pub struct Material {
    pub id: Uuid,
    pub formula: String,
    pub structure: Structure,
    pub properties: HashMap<String, Property>,
    pub metadata: Metadata,
}

pub trait ComputationMethod: Send + Sync {
    async fn calculate_energy(&self, material: &Material) -> Result<f64>;
    async fn calculate_forces(&self, material: &Material) -> Result<Array2<f64>>;
    fn cost_estimate(&self, material: &Material) -> Duration;
}
```

#### Milestone 1.3: ML Engine Básico (Semanas 6-8)

```rust
// Candle integration
- [ ] Model loading (GGUF, SafeTensors)
- [ ] Inference pipeline
- [ ] Batch processing
- [ ] GPU support (CUDA, Metal, Vulkan)

// Neural Network Potentials
- [ ] E(3)-equivariant GNN (SchNet, DimeNet)
- [ ] Universal NNP (MACE, NequIP style)
- [ ] Energy + Force prediction
- [ ] Uncertainty quantification
```

**Performance Targets:**
- 10,000 atoms energy calc: < 100ms (GPU)
- Batch inference (100 structures): < 1s
- Memory efficiency: < 500MB per model

---

### Fase 2: Multi-LLM Integration (Semanas 9-14) - ALTA PRIORIDAD

#### Milestone 2.1: LLM Provider Abstraction (Semanas 9-10)

```rust
pub trait LLMProvider: Send + Sync {
    async fn complete(&self, prompt: &str, params: CompletionParams)
        -> Result<Completion>;
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
    fn cost_per_token(&self) -> f64;
    fn max_tokens(&self) -> usize;
    fn supports_streaming(&self) -> bool;
}

// Implementaciones
- [ ] OpenAI provider (GPT-4, GPT-3.5)
- [ ] Anthropic provider (Claude-3.5, Claude-3)
- [ ] Google provider (Gemini Pro, Ultra)
- [ ] Mistral provider (API + local)
- [ ] Local provider (llm crate, candle)
```

#### Milestone 2.2: Smart Router & Fallback (Semanas 11-12)

```rust
pub struct LLMRouter {
    providers: Vec<Box<dyn LLMProvider>>,
    fallback_chain: Vec<String>,
    circuit_breakers: HashMap<String, CircuitBreaker>,
}

// Features
- [ ] Cost-based routing (cheapest first)
- [ ] Speed-based routing (fastest for latency-critical)
- [ ] Availability-based routing (health checks)
- [ ] Automatic failover (< 100ms)
- [ ] Circuit breaker pattern
- [ ] Rate limiting per provider
- [ ] Token quota management
```

**Fallback Chain Configuración:**
```yaml
llm_routing:
  primary: "gpt-4-turbo"
  fallbacks:
    - provider: "claude-3-5-sonnet"
      threshold: "5s timeout"
    - provider: "gemini-pro"
      threshold: "rate_limit"
    - provider: "mixtral-8x7b-local"
      threshold: "cost > $1"
    - provider: "phi2-local"
      threshold: "offline_mode"
    - provider: "tinyllama-local"
      threshold: "last_resort"
```

#### Milestone 2.3: Local Model Integration (Semanas 13-14)

```rust
// Local model support
- [ ] GGUF format loading (llm crate)
- [ ] Candle inference for small models
- [ ] Model quantization (4-bit, 8-bit)
- [ ] CPU-optimized inference
- [ ] Memory-mapped models (mmap)

// Supported models
- [ ] TinyLlama-1.1B (fast, 1GB RAM)
- [ ] Phi2-2.7B (quality, 3GB RAM)
- [ ] Mistral-7B (best quality, 8GB RAM)
- [ ] Mixtral-8x7B-MoE (expert, 32GB RAM)
```

**Performance Targets Local:**
- TinyLlama: 50 tokens/s (CPU)
- Phi2: 30 tokens/s (CPU)
- Mistral-7B: 100 tokens/s (GPU)

---

### Fase 3: Computation Engine (Semanas 15-22)

#### Milestone 3.1: Molecular Dynamics Engine (Semanas 15-17)

```rust
- [ ] Custom MD engine (NVE, NVT, NPT)
- [ ] Integrators (Verlet, Velocity Verlet, Leapfrog)
- [ ] Thermostats (Nosé-Hoover, Berendsen, Langevin)
- [ ] Barostats (Parrinello-Rahman)
- [ ] Neighbor lists (Verlet lists)
- [ ] Parallel force calculation (rayon)
- [ ] GPU kernels (CUDA via cudarc)
- [ ] LAMMPS FFI bridge
```

#### Milestone 3.2: Property Calculators (Semanas 18-20)

```rust
// Core properties
- [ ] Formation energy
- [ ] Band structure
- [ ] Density of states (DOS)
- [ ] Elastic constants
- [ ] Phonon dispersion
- [ ] Thermal conductivity
- [ ] Dielectric properties

// Analysis tools
- [ ] Radial distribution function (RDF)
- [ ] Mean squared displacement (MSD)
- [ ] Structure factor
- [ ] Pair correlation
```

#### Milestone 3.3: DFT Bridges (Semanas 21-22)

```rust
// External DFT connectors
- [ ] VASP input/output parser
- [ ] Quantum ESPRESSO connector
- [ ] GPAW bridge (via PyO3)
- [ ] Job scheduling interface
- [ ] Result parsers
```

---

### Fase 4: API & Services (Semanas 23-28)

#### Milestone 4.1: REST API (Semanas 23-24)

```rust
// Axum-based REST API
- [ ] Material CRUD endpoints
- [ ] Calculation submission
- [ ] Query endpoints
- [ ] Authentication middleware
- [ ] Rate limiting
- [ ] OpenAPI documentation (utoipa)
```

#### Milestone 4.2: GraphQL API (Semanas 25-26)

```rust
// async-graphql integration
- [ ] Schema definition
- [ ] Resolvers for complex queries
- [ ] Subscriptions (real-time)
- [ ] DataLoader pattern
- [ ] Query complexity limiting
```

#### Milestone 4.3: gRPC Services (Semanas 27-28)

```rust
// Tonic-based gRPC
- [ ] Service definitions (.proto)
- [ ] Compute service (simulations)
- [ ] Database service (queries)
- [ ] Streaming support
- [ ] Load balancing
```

---

### Fase 5: Monitoring & Production (Semanas 29-32)

#### Milestone 5.1: Observability (Semanas 29-30)

```rust
// Prometheus metrics
- [ ] Request rates, latencies
- [ ] Computation metrics
- [ ] Database performance
- [ ] LLM usage statistics
- [ ] Custom business metrics

// Distributed tracing
- [ ] OpenTelemetry integration
- [ ] Span creation
- [ ] Context propagation
- [ ] Jaeger export

// Logging
- [ ] Structured logging (tracing)
- [ ] Log levels (debug, info, warn, error)
- [ ] Log aggregation
- [ ] Audit logging
```

#### Milestone 5.2: Deployment (Semanas 31-32)

```rust
// Docker
- [ ] Multi-stage builds
- [ ] Optimized images (< 50MB)
- [ ] Docker Compose for dev

// Kubernetes
- [ ] Deployments
- [ ] Services
- [ ] ConfigMaps/Secrets
- [ ] HPA (autoscaling)
- [ ] Ingress

// CI/CD
- [ ] GitHub Actions workflows
- [ ] Automated testing
- [ ] Security scanning (cargo-audit)
- [ ] Container scanning
- [ ] Automated deployment
```

---

## 🛡️ PROCESO DE ASEGURAMIENTO DE CALIDAD

### Testing Strategy

#### 1. Unit Testing

```rust
// Coverage target: 90%+
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_material_creation() {
        let material = Material::new("Fe2O3");
        assert_eq!(material.formula, "Fe2O3");
    }

    #[test]
    fn test_energy_calculation() {
        // Property-based testing
        quickcheck(|atoms: Vec<Atom>| {
            let energy = calculate_energy(&atoms);
            energy.is_finite() && energy < 0.0
        });
    }
}
```

**Tools:**
- `cargo test` - Standard test runner
- `cargo-nextest` - Faster test execution
- `proptest` - Property-based testing
- `quickcheck` - QuickCheck-style testing

#### 2. Integration Testing

```rust
// tests/integration/database_test.rs
#[tokio::test]
async fn test_material_crud_workflow() {
    let db = setup_test_db().await;

    // Create
    let material = db.create_material(sample_material()).await?;

    // Read
    let retrieved = db.get_material(material.id).await?;
    assert_eq!(retrieved.formula, material.formula);

    // Update
    db.update_material(material.id, updates).await?;

    // Delete
    db.delete_material(material.id).await?;

    teardown_test_db(db).await;
}
```

#### 3. Performance Testing

```rust
// benches/compute_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_energy_calculation(c: &mut Criterion) {
    let material = create_benchmark_material(1000); // 1000 atoms

    c.bench_function("energy_calc_1k_atoms", |b| {
        b.iter(|| {
            calculate_energy(black_box(&material))
        })
    });
}

criterion_group!(benches, benchmark_energy_calculation);
criterion_main!(benches);
```

**Performance Targets:**
| Operation | Target | Measurement |
|-----------|--------|-------------|
| Material DB insert | < 1ms | p95 |
| Energy calc (ML, 1K atoms) | < 10ms | p99 |
| Energy calc (ML, 10K atoms) | < 100ms | p99 |
| LLM routing decision | < 1ms | p95 |
| API response time | < 50ms | p95 |

#### 4. Load Testing

```bash
# Using drill for load testing
drill --benchmark load_test.yml --stats

# Or wrk
wrk -t12 -c400 -d30s --latency http://localhost:8080/api/materials
```

**Load Test Targets:**
- 10,000 req/s sustained
- < 100ms p99 latency
- Zero errors at 5,000 req/s

#### 5. Security Testing

```rust
// Security audit
- [ ] cargo audit (dependencies)
- [ ] cargo-deny (license compliance)
- [ ] clippy with security lints
- [ ] SAST tools (semgrep)
- [ ] Dependency scanning (Dependabot)

// Penetration testing
- [ ] SQL injection tests
- [ ] Authentication bypass tests
- [ ] Authorization tests
- [ ] Rate limiting tests
- [ ] Input validation tests
```

#### 6. Compatibility Testing

```rust
// MSRV testing
- [ ] Test on Rust 1.75.0 (MSRV)
- [ ] Test on Rust 1.82.0 (stable)
- [ ] Test on nightly (future-proofing)

// Platform testing
- [ ] Linux (x86_64, aarch64)
- [ ] macOS (Intel, Apple Silicon)
- [ ] Windows (MSVC, GNU)

// Database compatibility
- [ ] PostgreSQL 13, 14, 15, 16
- [ ] MongoDB 5.0, 6.0, 7.0
- [ ] Redis 6.x, 7.x
```

### Continuous Integration

```yaml
# .github/workflows/ci.yml
name: CI/CD Pipeline

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: ['1.75.0', 'stable', 'nightly']
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}

      - name: Run tests
        run: cargo test --all --all-features

      - name: Run clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Security audit
        run: cargo audit

      - name: Build
        run: cargo build --release

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin
      - name: Generate coverage
        run: cargo tarpaulin --out Xml --output-dir coverage/
      - name: Upload to codecov
        uses: codecov/codecov-action@v3

  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run benchmarks
        run: cargo bench --no-fail-fast
      - name: Upload benchmark results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: target/criterion/
```

### Quality Gates

**Pre-Merge Requirements:**
- ✅ All tests passing (unit + integration)
- ✅ Code coverage ≥ 85%
- ✅ No clippy warnings (with `-D warnings`)
- ✅ Formatted with rustfmt
- ✅ No security vulnerabilities (cargo audit)
- ✅ Benchmarks not regressed > 10%
- ✅ Documentation updated
- ✅ CHANGELOG updated

**Release Requirements:**
- ✅ All quality gates passed
- ✅ Performance benchmarks meet targets
- ✅ Load tests passed
- ✅ Security scan clean
- ✅ Documentation complete
- ✅ Migration guide (if breaking changes)

---

## 🔐 PARÁMETROS DE SEGURIDAD PARA ESCALAMIENTO

### Authentication & Authorization

```rust
// JWT-based authentication
pub struct AuthConfig {
    jwt_secret: String,              // 256-bit secret
    token_expiry: Duration,          // Default: 8 hours
    refresh_token_expiry: Duration,  // Default: 30 days
    mfa_required: bool,              // Multi-factor auth
}

// Role-Based Access Control (RBAC)
pub enum Role {
    Admin,          // Full access
    Researcher,     // Read + Compute
    ReadOnly,       // Read-only
}

pub struct Permission {
    resource: ResourceType,
    action: Action,    // Create, Read, Update, Delete, Execute
}

// Multi-tenant isolation
pub struct Tenant {
    id: Uuid,
    name: String,
    quota: ResourceQuota,  // CPU, Memory, Storage, API calls
    isolation_level: IsolationLevel,
}
```

### Data Encryption

```rust
// At rest
- [ ] AES-256-GCM for sensitive data
- [ ] Encrypted database columns (sqlcipher, pgcrypto)
- [ ] Encrypted file storage (S3 server-side encryption)
- [ ] Key rotation (monthly)

// In transit
- [ ] TLS 1.3 mandatory
- [ ] Certificate management (Let's Encrypt)
- [ ] mTLS for service-to-service
- [ ] Perfect Forward Secrecy (PFS)
```

### Input Validation

```rust
// All inputs validated
use validator::Validate;

#[derive(Validate, Deserialize)]
pub struct MaterialInput {
    #[validate(length(min = 1, max = 100))]
    formula: String,

    #[validate(range(min = 1, max = 10000))]
    num_atoms: usize,

    #[validate(email)]
    user_email: String,
}

// SQL injection prevention (SQLx compile-time checks)
sqlx::query!("SELECT * FROM materials WHERE id = $1", material_id)
    .fetch_one(&pool)
    .await?;
```

### Rate Limiting

```rust
pub struct RateLimiter {
    // Token bucket algorithm
    requests_per_second: u32,  // Default: 100
    burst_size: u32,           // Default: 200

    // Per-user limits
    user_limits: HashMap<Uuid, RateLimit>,

    // Per-tenant limits
    tenant_limits: HashMap<Uuid, RateLimit>,

    // Global limits
    global_limit: RateLimit,
}

// Implementation with Redis
- [ ] Distributed rate limiting
- [ ] Sliding window counter
- [ ] Token bucket per user/tenant
- [ ] 429 Too Many Requests responses
```

### Circuit Breaker Pattern

```rust
pub struct CircuitBreaker {
    state: CircuitState,
    failure_threshold: u32,      // Open after N failures
    timeout: Duration,           // Half-open after timeout
    success_threshold: u32,      // Close after N successes
}

pub enum CircuitState {
    Closed,      // Normal operation
    Open,        // Blocking requests
    HalfOpen,    // Testing if recovered
}

// Applied to:
- [ ] Database connections
- [ ] External API calls (LLMs)
- [ ] DFT calculations
- [ ] Microservice communication
```

### Audit Logging

```rust
pub struct AuditLog {
    timestamp: DateTime<Utc>,
    user_id: Option<Uuid>,
    tenant_id: Option<Uuid>,
    action: String,
    resource: String,
    resource_id: Option<Uuid>,
    status: AuditStatus,
    ip_address: IpAddr,
    user_agent: String,
    details: serde_json::Value,
}

// All operations logged:
- [ ] Authentication attempts
- [ ] Authorization decisions
- [ ] Data access (read/write)
- [ ] Configuration changes
- [ ] Computation submissions
- [ ] LLM API calls
- [ ] Database queries

// Immutable audit trail (append-only)
- [ ] PostgreSQL with append-only table
- [ ] Retention: 7 years (compliance)
- [ ] Export to S3 for long-term storage
```

### Secrets Management

```rust
// Never commit secrets to git
- [ ] Environment variables for local dev
- [ ] Kubernetes secrets for production
- [ ] HashiCorp Vault integration (future)
- [ ] AWS Secrets Manager (cloud)

// Secrets rotation
- [ ] Database passwords: monthly
- [ ] API keys: quarterly
- [ ] JWT secrets: quarterly
- [ ] Encryption keys: yearly
```

### Network Security

```yaml
# Kubernetes Network Policies
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: materials-simulato-r-policy
spec:
  podSelector:
    matchLabels:
      app: materials-simulato-r
  policyTypes:
    - Ingress
    - Egress
  ingress:
    - from:
      - podSelector:
          matchLabels:
            app: api-gateway
      ports:
        - protocol: TCP
          port: 8080
  egress:
    - to:
      - podSelector:
          matchLabels:
            app: postgres
      ports:
        - protocol: TCP
          port: 5432
```

### Compliance & Standards

```rust
// GDPR Compliance
- [ ] Data privacy by design
- [ ] Right to deletion (user data)
- [ ] Data portability
- [ ] Consent management
- [ ] Data minimization

// SOC 2 Type II
- [ ] Security controls
- [ ] Availability monitoring
- [ ] Processing integrity
- [ ] Confidentiality measures
- [ ] Privacy controls

// ISO 27001
- [ ] Information security management
- [ ] Risk assessment
- [ ] Incident response plan
- [ ] Business continuity
```

### Security Hardening

```rust
// Rust-specific
- [ ] No unsafe code in production (or minimized + audited)
- [ ] Deny warnings in CI
- [ ] Regular dependency updates
- [ ] cargo-audit in CI/CD
- [ ] SAST tools (clippy with security lints)

// Container hardening
- [ ] Distroless base images
- [ ] Non-root user
- [ ] Read-only filesystem
- [ ] Minimal attack surface
- [ ] Vulnerability scanning (Trivy, Grype)

// Infrastructure
- [ ] Principle of least privilege
- [ ] Defense in depth
- [ ] Regular security audits
- [ ] Penetration testing (annual)
- [ ] Bug bounty program (future)
```

---

## 📊 INTEGRACIÓN MULTI-LLM Y FALLBACK

### LLM Provider Matrix

| Provider | Model | Cost/1M tokens | Speed | Offline | Quality | Use Case |
|----------|-------|----------------|-------|---------|---------|----------|
| **OpenAI** | GPT-4 Turbo | $10/$30 | Medium | ❌ | ⭐⭐⭐⭐⭐ | Complex reasoning |
| **OpenAI** | GPT-3.5 Turbo | $0.50/$1.50 | Fast | ❌ | ⭐⭐⭐⭐ | General tasks |
| **Anthropic** | Claude-3.5 Sonnet | $3/$15 | Fast | ❌ | ⭐⭐⭐⭐⭐ | Scientific analysis |
| **Anthropic** | Claude-3 Haiku | $0.25/$1.25 | Very Fast | ❌ | ⭐⭐⭐ | Simple tasks |
| **Google** | Gemini Pro | $0.50/$1.50 | Fast | ❌ | ⭐⭐⭐⭐ | Multi-modal |
| **Mistral** | Mixtral-8x7B | $0.70/$0.70 | Medium | ✅ | ⭐⭐⭐⭐ | Best local |
| **Mistral** | Mistral-7B | Free (local) | Fast | ✅ | ⭐⭐⭐ | Local fallback |
| **Microsoft** | Phi2 (2.7B) | Free (local) | Very Fast | ✅ | ⭐⭐⭐ | Offline mode |
| **TinyLlama** | TinyLlama-1.1B | Free (local) | Ultra Fast | ✅ | ⭐⭐ | Last resort |

### Smart Routing Logic

```rust
pub struct LLMRouter {
    config: RouterConfig,
    providers: HashMap<String, Box<dyn LLMProvider>>,
    circuit_breakers: HashMap<String, CircuitBreaker>,
    metrics: Arc<Metrics>,
}

pub struct RouterConfig {
    primary_provider: String,
    routing_strategy: RoutingStrategy,
    fallback_chain: Vec<FallbackRule>,
    cost_budget: Option<f64>,  // Max $ per request
    latency_sla: Option<Duration>,  // Max latency
}

pub enum RoutingStrategy {
    CostOptimized,      // Cheapest first
    LatencyOptimized,   // Fastest first
    QualityOptimized,   // Best quality first
    BalancedCostSpeed,  // Balance cost and speed
    RoundRobin,         // Distribute load
}

pub struct FallbackRule {
    condition: FallbackCondition,
    target_provider: String,
    max_retries: u32,
}

pub enum FallbackCondition {
    Timeout(Duration),
    RateLimit,
    CostExceeded(f64),
    Offline,
    CircuitOpen,
    ErrorRate(f64),  // Error rate threshold
}
```

### Implementación del Router

```rust
impl LLMRouter {
    pub async fn complete(&self, request: CompletionRequest)
        -> Result<Completion>
    {
        let mut fallback_chain = self.build_fallback_chain(&request);

        for (attempt, provider_name) in fallback_chain.iter().enumerate() {
            // Check circuit breaker
            if let Some(cb) = self.circuit_breakers.get(provider_name) {
                if cb.is_open() {
                    tracing::warn!(
                        provider = provider_name,
                        "Circuit breaker open, skipping"
                    );
                    continue;
                }
            }

            let provider = self.providers.get(provider_name)
                .ok_or_else(|| anyhow!("Provider not found: {}", provider_name))?;

            // Record attempt
            self.metrics.increment_llm_attempt(provider_name);

            let start = Instant::now();

            match timeout(
                self.config.latency_sla.unwrap_or(Duration::from_secs(30)),
                provider.complete(&request.prompt, request.params.clone())
            ).await {
                Ok(Ok(completion)) => {
                    let latency = start.elapsed();

                    // Record success
                    self.metrics.record_llm_success(
                        provider_name,
                        latency,
                        completion.tokens_used,
                    );

                    // Update circuit breaker
                    if let Some(cb) = self.circuit_breakers.get_mut(provider_name) {
                        cb.record_success();
                    }

                    tracing::info!(
                        provider = provider_name,
                        attempt = attempt,
                        latency_ms = latency.as_millis(),
                        tokens = completion.tokens_used,
                        "LLM completion successful"
                    );

                    return Ok(completion);
                }

                Ok(Err(e)) => {
                    // Record failure
                    self.metrics.record_llm_failure(provider_name, &e);

                    // Update circuit breaker
                    if let Some(cb) = self.circuit_breakers.get_mut(provider_name) {
                        cb.record_failure();
                    }

                    tracing::warn!(
                        provider = provider_name,
                        attempt = attempt,
                        error = ?e,
                        "LLM request failed, trying fallback"
                    );
                }

                Err(_timeout) => {
                    tracing::warn!(
                        provider = provider_name,
                        attempt = attempt,
                        "LLM request timed out, trying fallback"
                    );

                    self.metrics.increment_llm_timeout(provider_name);
                }
            }
        }

        Err(anyhow!("All LLM providers failed"))
    }

    fn build_fallback_chain(&self, request: &CompletionRequest)
        -> Vec<String>
    {
        match self.config.routing_strategy {
            RoutingStrategy::CostOptimized => {
                self.providers_by_cost(request)
            }
            RoutingStrategy::LatencyOptimized => {
                self.providers_by_speed(request)
            }
            RoutingStrategy::QualityOptimized => {
                self.providers_by_quality(request)
            }
            RoutingStrategy::BalancedCostSpeed => {
                self.providers_balanced(request)
            }
            RoutingStrategy::RoundRobin => {
                self.providers_round_robin()
            }
        }
    }

    fn providers_by_cost(&self, request: &CompletionRequest) -> Vec<String> {
        let mut providers: Vec<_> = self.providers.iter()
            .map(|(name, provider)| {
                let cost = provider.estimate_cost(request);
                (name.clone(), cost)
            })
            .collect();

        providers.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        providers.into_iter().map(|(name, _)| name).collect()
    }
}
```

### Fallback Chain Configuration

```yaml
# config/llm.yml
llm_routing:
  strategy: "balanced_cost_speed"

  primary_provider: "claude-3-5-sonnet"

  fallback_rules:
    - condition:
        type: "timeout"
        duration: "5s"
      target: "gpt-4-turbo"
      max_retries: 2

    - condition:
        type: "rate_limit"
      target: "gemini-pro"
      max_retries: 3

    - condition:
        type: "cost_exceeded"
        max_cost: 0.10  # $0.10 per request
      target: "gpt-3.5-turbo"
      max_retries: 2

    - condition:
        type: "circuit_open"
      target: "mixtral-8x7b-local"
      max_retries: 1

    - condition:
        type: "offline"
      target: "phi2-local"
      max_retries: 1

    - condition:
        type: "last_resort"
      target: "tinyllama-local"
      max_retries: 1

  circuit_breaker:
    failure_threshold: 5
    timeout_seconds: 60
    success_threshold: 2

  cost_budget:
    daily_limit: 100.0  # $100/day
    per_request_limit: 1.0  # $1/request
    alert_threshold: 0.8  # Alert at 80%

  latency_sla:
    p50: "1s"
    p95: "3s"
    p99: "5s"
    timeout: "30s"
```

### Local Model Management

```rust
pub struct LocalModelManager {
    models: HashMap<String, LoadedModel>,
    model_cache_dir: PathBuf,
    max_cache_size: usize,  // GB
}

pub struct LoadedModel {
    model_type: ModelType,
    quantization: Quantization,
    context_size: usize,
    memory_usage: usize,  // MB
}

pub enum ModelType {
    TinyLlama,
    Phi2,
    Mistral7B,
    Mixtral8x7B,
}

pub enum Quantization {
    F32,      // Full precision (slow, large)
    F16,      // Half precision
    Q8_0,     // 8-bit quantization
    Q4_K_M,   // 4-bit (fast, small)
    Q4_K_S,   // 4-bit (smallest)
}

impl LocalModelManager {
    pub async fn load_model(&mut self, config: ModelConfig)
        -> Result<String>
    {
        let model_path = self.download_if_needed(&config).await?;

        // Load model with llm crate or candle
        let model = match config.backend {
            Backend::LLM => {
                llm::load(
                    model_path,
                    config.quantization.into(),
                    llm::ModelParameters::default()
                )?
            }
            Backend::Candle => {
                // Candle-based loading
                self.load_with_candle(model_path, config)?
            }
        };

        let model_id = Uuid::new_v4().to_string();
        self.models.insert(model_id.clone(), model);

        Ok(model_id)
    }

    async fn download_if_needed(&self, config: &ModelConfig)
        -> Result<PathBuf>
    {
        let model_path = self.model_cache_dir
            .join(&config.model_name)
            .join(&config.quantization.to_string());

        if !model_path.exists() {
            tracing::info!(
                model = config.model_name,
                "Downloading model from HuggingFace Hub"
            );

            // Download from HF Hub
            self.download_from_hf_hub(config, &model_path).await?;
        }

        Ok(model_path)
    }
}
```

### Metrics & Observability

```rust
pub struct LLMMetrics {
    // Request metrics
    pub requests_total: Counter,
    pub requests_by_provider: HashMap<String, Counter>,

    // Latency metrics
    pub latency_histogram: Histogram,
    pub latency_by_provider: HashMap<String, Histogram>,

    // Cost metrics
    pub cost_total: Counter,
    pub cost_by_provider: HashMap<String, Counter>,

    // Failure metrics
    pub failures_total: Counter,
    pub failures_by_provider: HashMap<String, Counter>,
    pub timeout_count: Counter,

    // Circuit breaker state
    pub circuit_breaker_state: Gauge,

    // Token usage
    pub tokens_used_total: Counter,
    pub tokens_by_provider: HashMap<String, Counter>,
}

// Prometheus metrics export
impl LLMMetrics {
    pub fn register(registry: &Registry) -> Result<Self> {
        // Implementation...
    }

    pub fn record_request(&self, provider: &str, latency: Duration,
                          tokens: usize, cost: f64) {
        self.requests_total.inc();
        self.requests_by_provider.get(provider).unwrap().inc();

        self.latency_histogram.observe(latency.as_secs_f64());
        self.latency_by_provider.get(provider).unwrap()
            .observe(latency.as_secs_f64());

        self.cost_total.inc_by(cost);
        self.cost_by_provider.get(provider).unwrap().inc_by(cost);

        self.tokens_used_total.inc_by(tokens as f64);
        self.tokens_by_provider.get(provider).unwrap()
            .inc_by(tokens as f64);
    }
}
```

---

## 📈 MÉTRICAS DE ÉXITO

### Performance Metrics

| Métrica | Python (baseline) | Rust Target | Mejora |
|---------|-------------------|-------------|--------|
| **API Latency (p95)** | 200ms | 20ms | 10x |
| **Energy calc (ML, 1K atoms)** | 100ms | 10ms | 10x |
| **Energy calc (ML, 10K atoms)** | 1s | 100ms | 10x |
| **DB insert (1K materials)** | 10s | 1s | 10x |
| **DB query (complex)** | 100ms | 10ms (cached) | 10x |
| **Memory usage** | 500MB | 50MB | 10x |
| **Startup time** | 5s | 100ms | 50x |
| **CPU efficiency** | 60% | 95% | 1.6x |

### LLM Integration Metrics

| Métrica | Target | Measurement |
|---------|--------|-------------|
| **Routing decision** | < 1ms | p95 |
| **Failover time** | < 100ms | p99 |
| **Success rate (all providers)** | > 99.9% | Monthly |
| **Cost per 1M tokens** | < $5 (average) | Monthly |
| **Offline availability** | 100% | Always |

### Reliability Metrics

| Métrica | Target |
|---------|--------|
| **Uptime** | 99.9% |
| **MTBF** (Mean Time Between Failures) | > 720 hours |
| **MTTR** (Mean Time To Recovery) | < 5 minutes |
| **Error rate** | < 0.1% |
| **Circuit breaker effectiveness** | > 95% failures prevented |

---

## 🎯 CONCLUSIÓN

**Materials-Simulato-R** representa una refactorización completa y enterprise de Materials-SimPro a Rust, con los siguientes pilares:

✅ **Performance**: 10-100x más rápido que Python
✅ **Multi-LLM**: Integración transparente con fallback inteligente
✅ **Type Safety**: Rust garantiza seguridad en compile-time
✅ **Escalabilidad**: Arquitectura basada en AION-R enterprise
✅ **Procesos Centrales**: 100% operativos antes de cloud/security

### Próximos Pasos Inmediatos

1. **Semana 1-2**: Setup workspace, CI/CD, estándares
2. **Semana 3-8**: Database layer + Core types (CRÍTICO)
3. **Semana 9-14**: Multi-LLM integration (ALTA PRIORIDAD)
4. **Semana 15+**: Computation engine, API, deployment

---

**Versión:** 1.0.0
**Estado:** ✅ APROBADO PARA IMPLEMENTACIÓN
**Inicio:** 2025-11-07
**Rust MSRV:** 1.75.0
**Rust Target:** 1.82.0+

---

*"Rust: Performance without compromise. Science without limits."*

🦀 **¡Transformando la ciencia computacional con Rust!** 🚀
