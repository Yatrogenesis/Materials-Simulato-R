# Vectorización Asintótica con Atractores Caóticos para Compresión

**Autor**: Francisco Molina Burgos
**ORCID**: 0009-0008-6093-8267
**Fecha**: 2025-11-21
**Tipo**: Investigación Teórica Avanzada

---

## Concepto Fundamental

La idea es explotar la **estructura fractal latente** en embeddings de alta dimensionalidad mediante:

1. **Aproximación esférica**: Proyección en variedades esféricas
2. **Atractores caóticos**: Modelar trayectorias como sistemas dinámicos
3. **Vectorización asintótica**: Convergencia hacia estados estables del atractor

### Hipótesis Central

> Los embeddings conversacionales forman trayectorias en espacios de alta dimensionalidad que pueden ser **aproximadas por atractores caóticos de dimensión fractal baja**, permitiendo compresión extrema mediante codificación de parámetros del atractor en vez de vectores individuales.

---

## 1. Fundamentos Matemáticos

### 1.1 Dimensión Fractal de Embeddings

**Teorema de Takens** (1981):
Para un sistema dinámico con atractor de dimensión $d_A$, podemos reconstruir el atractor desde una serie temporal en un espacio de embedding de dimensión $m > 2d_A + 1$.

**Aplicación a embeddings ML**:
```
Embeddings 768D → Trayectoria en manifold de dimensión ~d_A
donde d_A << 768 (típicamente d_A ≈ 10-50)
```

**Dimensión de correlación** (Grassberger-Procaccia):
```
D_2 = lim_{r→0} log(C(r)) / log(r)

donde C(r) = lim_{N→∞} (1/N²) Σ Θ(r - ||v_i - v_j||)
```

**Medición empírica**:
```rust
fn estimate_correlation_dimension(vectors: &[Vec<f32>]) -> f64 {
    let mut correlation_sums = Vec::new();
    let radii = [0.01, 0.02, 0.05, 0.1, 0.2, 0.5];

    for &r in &radii {
        let mut count = 0;
        for i in 0..vectors.len() {
            for j in (i+1)..vectors.len() {
                if euclidean_distance(&vectors[i], &vectors[j]) < r {
                    count += 1;
                }
            }
        }
        let c_r = count as f64 / (vectors.len() * vectors.len()) as f64;
        correlation_sums.push((r, c_r));
    }

    // Regresión log-log para estimar D_2
    linear_regression_log_log(&correlation_sums)
}
```

### 1.2 Atractores Extraños en Embeddings

**Atractor de Lorenz** (ejemplo clásico):
```
dx/dt = σ(y - x)
dy/dt = x(ρ - z) - y
dz/dt = xy - βz
```

**Para embeddings conversacionales**:
```
v_{t+1} = F(v_t, c_t)

donde:
- v_t: embedding en tiempo t
- c_t: contexto (entrada usuario)
- F: función de transición (modelo neural)
```

**Propiedad clave**: Si F tiene estructura recurrente (LSTM, GRU), puede generar **atractores caóticos** en espacio de embeddings.

### 1.3 Aproximación Esférica Asintótica

**Proyección en esfera unitaria**:
```
v̂ = v / ||v||  (normalización)
```

**Sistema dinámico en S^{n-1}** (esfera n-dimensional):
```
θ_{t+1} = Φ(θ_t, ω_t)

donde θ ∈ [0, π]^{n-1} son ángulos esféricos
```

**Ventaja**: Espacio compacto → atractores bien definidos

---

## 2. Metodología Propuesta

### Fase 1: Identificación del Atractor

**Algoritmo**:
```rust
struct ChaoticAttractor {
    dimension: f64,           // Dimensión fractal D_2
    lyapunov_exponents: Vec<f64>,  // λ_i > 0 → caos
    embedding_dimension: usize,    // m mínimo para embedding
    attractor_params: Vec<f64>,    // Parámetros del modelo
}

fn identify_attractor(vectors: &[Vec<f32>]) -> ChaoticAttractor {
    // 1. Estimar dimensión de correlación
    let d_2 = estimate_correlation_dimension(vectors);

    // 2. Calcular exponentes de Lyapunov
    let lyapunov = estimate_lyapunov_exponents(vectors);

    // 3. Reconstrucción del atractor (Takens embedding)
    let m = (2.0 * d_2).ceil() as usize + 1;

    // 4. Ajustar modelo paramétrico del atractor
    let params = fit_attractor_model(vectors, m);

    ChaoticAttractor {
        dimension: d_2,
        lyapunov_exponents: lyapunov,
        embedding_dimension: m,
        attractor_params: params,
    }
}
```

### Fase 2: Codificación Basada en Atractor

**Idea central**: En vez de almacenar N vectores de 768D, almacenamos:
1. **Parámetros del atractor** (10-50 valores)
2. **Condiciones iniciales** (1 vector de 768D)
3. **Perturbaciones** por vector (deltas pequeños respecto a trayectoria del atractor)

**Compresión esperada**:
```
Original:        N × 768 × 4 bytes
Con atractor:    (50 + 768) × 4 + N × δ_size

donde δ_size << 768 × 4 (típicamente 10-100 bytes)

Para N=1000:
Original:        3,072,000 bytes
Con atractor:    3,272 + 100,000 = 103,272 bytes
Compresión:      ~30x
```

### Fase 3: Reconstrucción

**Algoritmo**:
```rust
fn reconstruct_from_attractor(
    attractor: &ChaoticAttractor,
    initial_condition: &Vec<f32>,
    perturbations: &[Vec<f32>]
) -> Vec<Vec<f32>> {
    let mut vectors = Vec::new();

    // Integrar sistema dinámico del atractor
    let mut state = initial_condition.clone();

    for (i, perturbation) in perturbations.iter().enumerate() {
        // Evolución del atractor
        state = evolve_attractor(&attractor, &state);

        // Aplicar perturbación
        let reconstructed = add_vectors(&state, perturbation);

        vectors.push(reconstructed);
    }

    vectors
}
```

---

## 3. Implementación Teórica

### 3.1 Modelo de Atractor: Lorenz Generalizado

Para embeddings de alta dimensionalidad:

```rust
struct GeneralizedLorenzAttractor {
    sigma: Vec<f64>,      // n parámetros σ
    rho: Vec<f64>,        // n parámetros ρ
    beta: Vec<f64>,       // n parámetros β
    coupling: Array2<f64>, // Matriz de acoplamiento n×n
}

impl GeneralizedLorenzAttractor {
    fn evolve(&self, state: &[f64], dt: f64) -> Vec<f64> {
        let n = state.len() / 3; // Grupos de 3 variables
        let mut new_state = state.to_vec();

        for i in 0..n {
            let x = state[i*3];
            let y = state[i*3 + 1];
            let z = state[i*3 + 2];

            // Ecuaciones de Lorenz generalizadas
            let dx = self.sigma[i] * (y - x);
            let dy = x * (self.rho[i] - z) - y;
            let dz = x * y - self.beta[i] * z;

            // Acoplamiento con otros subsistemas
            let coupling_x = self.coupling.row(i).dot(&state);

            new_state[i*3]     += (dx + coupling_x) * dt;
            new_state[i*3 + 1] += dy * dt;
            new_state[i*3 + 2] += dz * dt;
        }

        new_state
    }
}
```

### 3.2 Ajuste de Parámetros

**Optimización no-lineal**:
```rust
use nalgebra as na;

fn fit_attractor_parameters(
    vectors: &[Vec<f32>]
) -> GeneralizedLorenzAttractor {
    // 1. Proyectar a espacio de fase reducido (PCA)
    let reduced = pca_reduction(vectors, 30); // 768D → 30D

    // 2. Inicializar parámetros aleatorios
    let mut params = random_params(10); // 10 subsistemas × 3 vars

    // 3. Optimizar para minimizar error de reconstrucción
    let optimizer = LevenbergMarquardt::new();

    let final_params = optimizer.minimize(
        |p| reconstruction_error(p, &reduced),
        &params,
        1000 // iteraciones máximas
    );

    GeneralizedLorenzAttractor::from_params(final_params)
}

fn reconstruction_error(
    params: &[f64],
    actual: &[Vec<f32>]
) -> f64 {
    let attractor = GeneralizedLorenzAttractor::from_params(params);

    let mut state = actual[0].clone();
    let mut total_error = 0.0;

    for i in 1..actual.len() {
        state = attractor.evolve(&state, 0.01);
        let error = euclidean_distance(&state, &actual[i]);
        total_error += error * error;
    }

    total_error / actual.len() as f64
}
```

---

## 4. Ventajas y Desventajas

### Ventajas Teóricas

1. **Compresión extrema**: 30-100x posible si embeddings siguen atractor
2. **Interpolación natural**: Generación de estados intermedios
3. **Descubrimiento de estructura**: Revela dinámica subyacente
4. **Robustez a ruido**: Perturbaciones pequeñas absorbidas por atractor

### Desventajas Prácticas

1. **Complejidad computacional**: Ajuste de parámetros O(N² × M)
2. **Convergencia no garantizada**: Optimización no-convexa
3. **Asunción fuerte**: Requiere que embeddings REALMENTE formen atractor
4. **Pérdida de información**: Si datos no siguen atractor perfectamente

---

## 5. Validación Experimental

### Experimento 1: Medir Dimensión Fractal

```rust
fn experiment_fractal_dimension() {
    let vectors = generate_conversational_drift(1000, 768, 0.05);

    let d_2 = estimate_correlation_dimension(&vectors);

    println!("Dimensión de correlación: {:.2}", d_2);

    if d_2 < 50.0 {
        println!("✅ Embeddings tienen estructura de baja dimensión");
        println!("   Atractor viable con m = {}", (2.0 * d_2).ceil());
    } else {
        println!("❌ Dimensión demasiado alta para atractor simple");
    }
}
```

**Predicción**:
- Si d_2 < 30: Atractor de Lorenz generalizado puede funcionar
- Si 30 < d_2 < 100: Considerar modelos de mayor orden
- Si d_2 > 100: Método no viable (usar PCA primero)

### Experimento 2: Exponentes de Lyapunov

```rust
fn experiment_lyapunov() {
    let vectors = generate_conversational_drift(1000, 768, 0.05);

    let lambda = estimate_largest_lyapunov_exponent(&vectors);

    println!("λ_1 = {:.6}", lambda);

    if lambda > 0.0 {
        println!("✅ Sistema exhibe caos (λ > 0)");
        println!("   Atractor caótico presente");
    } else {
        println!("⚠️  Sistema no caótico (λ ≤ 0)");
        println!("   Considerar atractor periódico o cuasi-periódico");
    }
}
```

**Algoritmo de Rosenstein** para λ_max:
```rust
fn estimate_largest_lyapunov_exponent(vectors: &[Vec<f32>]) -> f64 {
    let tau = 10; // Delay de embedding
    let m = 5;    // Dimensión de embedding

    // Reconstruir espacio de fase con delay embedding
    let mut phase_space = Vec::new();
    for i in 0..(vectors.len() - m*tau) {
        let mut point = Vec::new();
        for j in 0..m {
            point.extend(&vectors[i + j*tau]);
        }
        phase_space.push(point);
    }

    // Encontrar vecinos cercanos
    let mut divergences = Vec::new();

    for (i, point) in phase_space.iter().enumerate() {
        // Buscar vecino más cercano (con separación temporal)
        let nearest = find_nearest_neighbor(&phase_space, i, 10);

        // Seguir divergencia en el tiempo
        let mut log_divergence = Vec::new();
        for dt in 1..50 {
            if i + dt < phase_space.len() && nearest + dt < phase_space.len() {
                let dist = euclidean_distance(
                    &phase_space[i + dt],
                    &phase_space[nearest + dt]
                );
                if dist > 1e-10 {
                    log_divergence.push(dist.ln());
                }
            }
        }

        divergences.push(log_divergence);
    }

    // Regresión lineal sobre log(divergence) vs tiempo
    average_slope(&divergences)
}
```

---

## 6. Comparación con Otros Métodos

| Aspecto | Atractor Caótico | PCA/KLT | Product Quantization | ANS |
|---------|------------------|---------|---------------------|-----|
| **Compresión** | 30-100x | 6x | 3.7-64x | 15-20x |
| **Pérdida** | 1-10% | <1% | 1-5% | 0% |
| **Complejidad** | O(N² × M) | O(D³) | O(NK log K) | O(N) |
| **Asunciones** | Estructura atractor | Linealidad | Clustering | Distribución |
| **Interpretabilidad** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐ | ⭐ |

**Ventaja única**: Revela **dinámica subyacente** del sistema

---

## 7. Aplicaciones Específicas

### Caso 1: Memoria Conversacional (Lirasion)

**Escenario**:
- Conversación larga (1000+ intercambios)
- Embeddings evolucionan gradualmente
- Alta similitud consecutiva

**Aplicabilidad**: ⭐⭐⭐⭐⭐ EXCELENTE
- Conversación = trayectoria en espacio semántico
- Cambios de tema = perturbaciones del atractor
- Contexto = parámetros de control

**Implementación**:
```rust
struct ConversationalAttractor {
    base_attractor: GeneralizedLorenzAttractor,
    context_modulation: Vec<f64>, // Parámetros por tema
}

impl ConversationalAttractor {
    fn compress_conversation(&self, embeddings: &[Vec<f32>]) -> CompressedMemory {
        // 1. Identificar cambios de tema (bifurcaciones)
        let topic_changes = detect_bifurcations(embeddings);

        // 2. Ajustar atractor por segmento
        let segments = segment_by_topics(embeddings, &topic_changes);

        // 3. Almacenar solo parámetros + perturbaciones
        CompressedMemory {
            attractor: self.base_attractor.clone(),
            topic_params: self.extract_topic_params(&segments),
            perturbations: compute_perturbations(&segments),
        }
    }
}
```

### Caso 2: Series Temporales de Embeddings

**Escenario**:
- Embeddings generados por modelo estable
- Datos con estructura temporal
- Periodicidad o quasi-periodicidad

**Aplicabilidad**: ⭐⭐⭐⭐ MUY BUENA

---

## 8. Investigación Relacionada

### Fractal Dimensionality Reduction (Traina, 2000)

**FDR Algorithm**:
- Usa dimensión fractal para selección de features
- Reduce dimensionalidad preservando estructura fractal

**Aplicación a embeddings**:
```rust
fn fdr_compress(vectors: &[Vec<f32>], target_dim: usize) -> Vec<Vec<f32>> {
    // 1. Calcular dimensión fractal por feature
    let fractal_dims = compute_feature_fractal_dims(vectors);

    // 2. Seleccionar features con mayor dim fractal
    let selected_features = select_top_k_features(&fractal_dims, target_dim);

    // 3. Proyectar
    project_to_features(vectors, &selected_features)
}
```

### Manifold Learning + Attractors

**Isomap + Atractor**:
1. Isomap encuentra manifold geodésico
2. Ajustar atractor sobre manifold reducido
3. Compresión sobre espacio de atractor

---

## 9. Roadmap de Implementación

### Fase 1: Validación Empírica (1 semana)
- [ ] Implementar estimación de D_2
- [ ] Medir dimensión fractal en datasets reales
- [ ] Calcular exponentes de Lyapunov
- [ ] Determinar si atractor existe

### Fase 2: Prototipo Simple (2 semanas)
- [ ] Implementar Lorenz generalizado 3D
- [ ] Ajustar parámetros con Levenberg-Marquardt
- [ ] Comprimir/descomprimir 100 vectores
- [ ] Medir compresión y pérdida

### Fase 3: Escalado (3-4 semanas)
- [ ] Generalizar a n-dimensional
- [ ] Optimización GPU para ajuste
- [ ] Compresión híbrida (Atractor + ANS para perturbaciones)
- [ ] Benchmarks comparativos

### Fase 4: Integración (2 semanas)
- [ ] Integrar en Lirasion
- [ ] API de compresión conversacional
- [ ] Tests de regresión
- [ ] Documentación

**Tiempo total estimado**: 8-10 semanas

---

## 10. Conclusiones

### Evaluación del Enfoque

**Viabilidad Técnica**: ⭐⭐⭐ (Moderada)
- Requiere validar existencia de atractor primero
- Complejidad computacional alta
- Riesgo de convergencia a mínimos locales

**Potencial de Compresión**: ⭐⭐⭐⭐⭐ (Excelente)
- Si atractor existe: 30-100x posible
- Mejor que métodos convencionales
- Compresión + interpretabilidad

**Aplicabilidad a Lirasion**: ⭐⭐⭐⭐ (Muy buena)
- Memoria conversacional = caso de uso ideal
- Estructura temporal natural
- Valor agregado: descubrimiento de patrones

### Recomendación

1. **Corto plazo**: Implementar Delta + ANS (método probado, 15x)
2. **Mediano plazo**: Experimento de validación de atractores
3. **Largo plazo**: Si validación exitosa, implementar compresión por atractor

**NO iniciar implementación completa** hasta validar que:
- D_2 < 50 (dimensión fractal manejable)
- λ_max > 0 (comportamiento caótico)
- Error de reconstrucción < 5%

---

## Referencias

1. **Takens, F.** (1981). "Detecting strange attractors in turbulence". Dynamical Systems and Turbulence, Lecture Notes in Mathematics, vol 898.

2. **Grassberger, P. & Procaccia, I.** (1983). "Characterization of Strange Attractors". Physical Review Letters 50: 346–349.

3. **Rosenstein, M. T., Collins, J. J., De Luca, C. J.** (1993). "A practical method for calculating largest Lyapunov exponents from small data sets". Physica D.

4. **Traina, C., et al.** (2000). "Fast Feature Selection using Fractal Dimension". XV Brazilian Symposium on Databases.

5. **Kantz, H. & Schreiber, T.** (2003). "Nonlinear Time Series Analysis". Cambridge University Press.

---

**Status**: 🔬 Investigación teórica - Requiere validación experimental antes de implementación
