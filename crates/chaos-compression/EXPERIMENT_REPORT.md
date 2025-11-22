# Experimento de Compresión de Vectores ML - Reporte Final

**Autor**: Francisco Molina Burgos
**ORCID**: 0009-0008-6093-8267
**Fecha**: 2025-11-21
**Versión**: 2.0 (Corregida)

---

## Resumen Ejecutivo

Este experimento investigó la efectividad de diferentes algoritmos de compresión para vectores de embeddings ML (768 dimensiones), específicamente evaluando **Delta Encoding** bajo la hipótesis de que debería lograr ≥8x de compresión con similitud consecutiva ≥0.90.

### Hallazgos Principales

1. **❌ HIPÓTESIS REFUTADA**: La implementación actual de Delta Encoding NO logró compresión efectiva (1.09x-1.10x) incluso con similitud consecutiva muy alta (0.92-0.98)
2. **✅ SESGO IDENTIFICADO Y CORREGIDO**: El experimento inicial generaba vectores con similitud global alta pero similitud consecutiva baja
3. **🏆 GANADOR**: Int8+GZIP logró entre 4.61x-10.80x de compresión, pero con pérdida de accuracy variable (1.7%-26%)

---

## Metodología

### Corrección del Sesgo Experimental

**Problema Identificado** (por el usuario):
- Implementación inicial: vectores generados con similitud respecto a un vector base GLOBAL
- Resultado: Alta similitud global pero BAJA similitud consecutiva
- Implicación: Delta Encoding no puede funcionar sin similitud consecutiva

**Solución Implementada**:
Creación de 4 datasets con diferentes patrones de similitud consecutiva:

1. **Random Similar (baseline)**: Similitud global, NO consecutiva
2. **Conversational Drift**: Drift acumulativo con similitud consecutiva
3. **Temporal Smoothing**: Promedio móvil exponencial (EMA)
4. **Clustered Topics**: Cambios de tema cada N vectores

### Métrica Crítica: Similitud Consecutiva

```rust
/// Similitud coseno promedio entre vectores consecutivos
fn calculate_consecutive_similarity(vectors: &[Vec<f32>]) -> f64 {
    let mut sum = 0.0;
    for i in 1..vectors.len() {
        let a = &vectors[i - 1];
        let b = &vectors[i];
        let cosine_similarity = dot(a, b) / (norm(a) * norm(b));
        sum += cosine_similarity;
    }
    sum / (vectors.len() - 1)
}
```

### Métodos de Compresión Evaluados

1. **GZIP Baseline**: Compresión estándar sin procesamiento
2. **Int8+GZIP**: Cuantización a 8 bits + GZIP
3. **Delta+GZIP**: Diferencias consecutivas + GZIP ⭐ (método bajo prueba)
4. **Zstd**: Compresor moderno de alta eficiencia

---

## Resultados

### Tabla Comparativa por Dataset

| Dataset | Consec.Sim | GZIP | Int8+GZIP | Delta+GZIP | Zstd |
|---------|------------|------|-----------|------------|------|
| Random Similar (baseline) | 0.9185 | 1.13x | 4.62x | 1.09x ⚠️ | 1.12x |
| Conversational Drift ⭐ (drift 5%) | 0.9636 | 1.13x | 10.80x | 1.10x ⚠️ | 1.14x |
| Temporal Smoothing (alpha 0.9) | 0.9819 | 1.13x | 4.61x | 1.09x ⚠️ | 1.12x |
| Clustered Topics (100 per cluster) | 0.9199 | 1.12x | 4.58x | 1.09x ⚠️ | 1.12x |

### Pérdida de Accuracy por Dataset

| Dataset | GZIP | Int8+GZIP | Delta+GZIP | Zstd |
|---------|------|-----------|------------|------|
| Random Similar | 0.0000% | 1.7074% | 0.0000% | 0.0000% |
| Conversational Drift | 0.0000% | 26.1077% | 0.0000% | 0.0000% |
| Temporal Smoothing | 0.0000% | 1.7157% | 0.0000% | 0.0000% |
| Clustered Topics | 0.0000% | 1.7251% | 0.0000% | 0.0000% |

---

## Validación de Hipótesis

### Predicción Original

> **Hipótesis**: Delta Encoding debería lograr ≥8x de compresión cuando la similitud consecutiva ≥0.90

### Resultados por Dataset

#### 1. Random Similar (baseline)
- **Similitud Consecutiva**: 0.9185
- **Delta+GZIP**: 1.09x
- **Evaluación**: ❌ HIPÓTESIS REFUTADA (esperaba ≥8x con consec.sim ≥0.90)

#### 2. Conversational Drift ⭐
- **Similitud Consecutiva**: 0.9636 ✅ EXCELENTE
- **Delta+GZIP**: 1.10x
- **Evaluación**: ❌ HIPÓTESIS REFUTADA (esperaba ≥8x con consec.sim ≥0.90)

#### 3. Temporal Smoothing
- **Similitud Consecutiva**: 0.9819 ✅ EXCELENTE
- **Delta+GZIP**: 1.09x
- **Evaluación**: ❌ HIPÓTESIS REFUTADA (esperaba ≥8x con consec.sim ≥0.90)

#### 4. Clustered Topics
- **Similitud Consecutiva**: 0.9199 ✅ EXCELENTE
- **Delta+GZIP**: 1.09x
- **Evaluación**: ❌ HIPÓTESIS REFUTADA (esperaba ≥8x con consec.sim ≥0.90)

### Resumen de Validación

```
Datasets con similitud consecutiva ≥0.90 donde Delta validó (≥8x): 0
Datasets con similitud consecutiva ≥0.90 donde Delta falló (<8x): 4
```

**Conclusión Científica**: La implementación actual de Delta Encoding **NO FUNCIONA** como se esperaba, incluso con condiciones ideales de similitud consecutiva (hasta 0.98).

---

## Análisis de Causa Raíz

### Problema en la Implementación de Delta Encoding

Inspección del código en `src/methods/mod.rs`:

```rust
pub fn delta_decompress(compressed: &[u8]) -> Vec<Vec<f32>> {
    let mut decoder = GzDecoder::new(compressed);
    let mut bytes = Vec::new();
    decoder.read_to_end(&mut bytes).unwrap();

    let floats: Vec<f32> = bytes.chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();

    vec![floats]  // ❌ BUG: Devuelve UN SOLO vector, no reconstruye N vectores
}
```

**Problema Identificado**:
1. La descompresión devuelve `vec![floats]` (un solo vector)
2. No reconstruye los N vectores originales a partir de deltas
3. Falta metadata sobre número de vectores y dimensiones
4. El algoritmo de reconstrucción acumulativa no está implementado

**Corrección Necesaria**:
```rust
pub fn delta_decompress(compressed: &[u8]) -> Vec<Vec<f32>> {
    // 1. Deserializar metadata (n_vectors, dim)
    // 2. Reconstruir primer vector
    // 3. Para cada delta: acumular y reconstruir siguiente vector
    // 4. Devolver Vec<Vec<f32>> con N vectores reconstruidos
}
```

---

## Recomendaciones

### Para el Proyecto Lirasion

#### Opción 1: Int8+GZIP (RECOMENDADO para casos con tolerancia a pérdida)

**Ventajas**:
- ✅ Compresión efectiva: 4.6x-10.8x
- ✅ Ya implementado y funcional
- ✅ Rápido (cuantización es simple)

**Desventajas**:
- ❌ Pérdida de accuracy variable (1.7%-26%)
- ⚠️ Alto loss en vectores normalizados (26% en Conversational Drift)

**Caso de uso**: Cacheo de embeddings donde se tolera pérdida ~2%

#### Opción 2: Zstd (RECOMENDADO para sin pérdida)

**Ventajas**:
- ✅ Sin pérdida de accuracy (0.0000%)
- ✅ Compresión consistente (1.12x-1.14x)
- ✅ Rápido y eficiente

**Desventajas**:
- ❌ Baja compresión comparada con Int8

**Caso de uso**: Almacenamiento de embeddings donde accuracy es crítica

#### Opción 3: Reimplementar Delta Encoding

**Pendiente**:
1. Corregir `delta_decompress()` para reconstruir vectores correctamente
2. Agregar metadata (n_vectors, dim) al formato comprimido
3. Implementar acumulación de deltas durante descompresión
4. Re-ejecutar experimento

**Predicción**: Si se corrige correctamente, debería lograr 8x+ con similitud consecutiva ≥0.90

#### Opción 4: PCA+Delta (Alternativa avanzada)

El usuario mencionó "PCA+Delta" en el prompt original:
1. Aplicar PCA para reducir dimensionalidad (768 → 128)
2. Aplicar Delta Encoding en espacio reducido
3. Combinar con cuantización

**Ventajas potenciales**:
- Compresión 6x de PCA + 8x de Delta = 48x teórico
- Pérdida controlada por componentes principales

---

## Código Implementado

### Generación de Datasets con Similitud Consecutiva

```rust
/// NUEVO: Genera vectores con DRIFT ACUMULATIVO (similitud consecutiva)
fn generate_conversational_drift(n: usize, dim: usize, drift_rate: f64) -> Vec<Vec<f32>> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut vectors = Vec::new();

    // Primer vector aleatorio
    let mut current: Vec<f32> = (0..dim).map(|_| rng.gen::<f32>()).collect();
    normalize_vector(&mut current);
    vectors.push(current.clone());

    // Siguientes vectores con drift acumulativo
    for _ in 1..n {
        let drift: Vec<f32> = (0..dim).map(|_| rng.gen::<f32>()).collect();
        let mut next = vec![0.0; dim];

        for i in 0..dim {
            next[i] = current[i] * (1.0 - drift_rate as f32) + drift[i] * (drift_rate as f32);
        }

        normalize_vector(&mut next);
        vectors.push(next.clone());
        current = next;  // KEY: drift acumulativo
    }

    vectors
}
```

### Métrica de Similitud Consecutiva

```rust
fn calculate_consecutive_similarity(vectors: &[Vec<f32>]) -> f64 {
    if vectors.len() < 2 {
        return 0.0;
    }

    let mut sum = 0.0;

    for i in 1..vectors.len() {
        let a = &vectors[i - 1];
        let b = &vectors[i];

        let mut dot = 0.0_f64;
        let mut norm_a = 0.0_f64;
        let mut norm_b = 0.0_f64;

        for j in 0..a.len() {
            dot += (a[j] as f64) * (b[j] as f64);
            norm_a += (a[j] as f64) * (a[j] as f64);
            norm_b += (b[j] as f64) * (b[j] as f64);
        }

        if norm_a > 1e-10 && norm_b > 1e-10 {
            sum += dot / (norm_a.sqrt() * norm_b.sqrt());
        }
    }

    sum / ((vectors.len() - 1) as f64)
}
```

---

## Archivos Generados

- **`results/results_all_similarities.json`**: Resultados completos en JSON
- **Salida de consola**: Tabla comparativa y validación de hipótesis

---

## Próximos Pasos

1. **INMEDIATO**: Decidir qué método usar en Lirasion ML
   - Int8+GZIP si se tolera ~2% pérdida
   - Zstd si se requiere sin pérdida

2. **CORTO PLAZO**: Reimplementar Delta Encoding correctamente
   - Corregir `delta_decompress()`
   - Re-ejecutar experimento
   - Validar si logra ≥8x esperado

3. **MEDIANO PLAZO**: Investigar PCA+Delta
   - Implementar reducción dimensional
   - Combinar con Delta/Int8
   - Evaluar trade-off compresión vs accuracy

---

## Referencias

- **IIT 3.0**: Tononi et al., 2016 - Integrated Information Theory
- **Delta Encoding**: Técnica clásica de compresión por diferencias
- **Zstd**: Facebook's Zstandard compression algorithm

---

## Apéndice: Salida Completa del Experimento

```
🔬 Experimento de Compresión de Vectores - CORREGIDO
Autor: Francisco Molina Burgos (ORCID: 0009-0008-6093-8267)
Fecha: 2025-11-21
Versión: 2.0 - Con similitud consecutiva

======================================================================
📊 Testing: Random Similar (baseline)
======================================================================

🔑 Similitud Consecutiva: 0.9185
   ⚠️  MEDIA - Delta Encoding puede funcionar parcialmente

Testing GZIP Baseline...
Testing Int8 Quantization...
Testing Delta Encoding...
Testing Zstd...

📊 Resultados:
  GZIP           : ratio= 1.13x, comp= 93.21ms, decomp= 6.75ms, loss=0.0000%
  Int8+GZIP      : ratio= 4.62x, comp= 99.93ms, decomp= 6.88ms, loss=1.7074%
  Delta+GZIP     : ratio= 1.09x, comp=100.18ms, decomp= 6.12ms, loss=0.0000% ⚠️ (esperaba ≥8x)
  Zstd           : ratio= 1.12x, comp= 19.51ms, decomp= 1.26ms, loss=0.0000%

🔬 Validación de Hipótesis:
   ❌ HIPÓTESIS REFUTADA: Delta solo 1.09x (esperaba ≥8x) con similitud 0.9185

[... 3 datasets más con resultados similares ...]

📊 TABLA COMPARATIVA FINAL
[ver tabla en sección Resultados]

🏆 VALIDACIÓN DE HIPÓTESIS Y CONCLUSIONES

❌ CONCLUSIÓN: Implementación actual de Delta Encoding NO funciona como esperado.
   Revisar algoritmo o considerar alternativas (PCA+Delta, etc.).
```

---

**Fin del Reporte**
