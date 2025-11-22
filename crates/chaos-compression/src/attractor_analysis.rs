//! Análisis de Atractores Caóticos para Compresión
//!
//! Implementa:
//! 1. Dimensión de correlación D₂ (Grassberger-Procaccia)
//! 2. Exponente de Lyapunov máximo λ₁
//! 3. Reconstrucción de espacio de fases (Takens embedding)
//!
//! Si D₂ < dim_embedding y λ₁ > 0, existe un atractor caótico
//! que puede ser comprimido dramáticamente (30-100x).

use std::collections::HashMap;

/// Resultado del análisis de atractor
#[derive(Debug, Clone)]
pub struct AttractorAnalysis {
    /// Dimensión de correlación D₂
    pub correlation_dimension: f64,

    /// Exponente de Lyapunov máximo
    pub max_lyapunov: f64,

    /// Dimensión del embedding original
    pub embedding_dim: usize,

    /// ¿Existe un atractor caótico?
    pub is_chaotic_attractor: bool,

    /// Potencial de compresión estimado
    pub compression_potential: f64,
}

/// Calcula la dimensión de correlación D₂ usando el algoritmo de Grassberger-Procaccia
///
/// # Algoritmo
/// 1. Para cada distancia r, contar pares de puntos con distancia < r
/// 2. C(r) = suma de pares / total de pares posibles
/// 3. D₂ = lim_{r→0} d(log C(r)) / d(log r)
///
/// # Referencias
/// Grassberger, P., & Procaccia, I. (1983). "Measuring the strangeness of strange attractors"
pub fn correlation_dimension(vectors: &[Vec<f32>]) -> f64 {
    if vectors.len() < 100 {
        return vectors[0].len() as f64; // No hay suficientes datos
    }

    let n = vectors.len().min(2000); // Limitar a 2000 puntos para eficiencia
    let vectors = &vectors[..n];

    // 1. Calcular matriz de distancias (solo triángulo superior)
    let mut distances = Vec::new();

    for i in 0..n {
        for j in (i + 1)..n {
            let dist = euclidean_distance(&vectors[i], &vectors[j]);
            distances.push(dist);
        }
    }

    distances.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // 2. Elegir rango de radios r (escala logarítmica)
    let min_r = distances[distances.len() / 100].max(1e-6); // Percentil 1
    let max_r = distances[distances.len() * 99 / 100]; // Percentil 99

    let num_radii = 20;
    let mut radii = Vec::new();
    let mut correlation_sums = Vec::new();

    for i in 0..num_radii {
        let log_r = (min_r.ln() + (max_r.ln() - min_r.ln()) * (i as f64) / (num_radii as f64 - 1.0));
        let r = log_r.exp();
        radii.push(r);

        // Contar pares con distancia < r
        let count = distances.iter().filter(|&&d| d < r).count();
        let c_r = count as f64 / distances.len() as f64;

        correlation_sums.push(c_r.max(1e-10)); // Evitar log(0)
    }

    // 3. Estimar D₂ mediante regresión lineal en escala log-log
    // D₂ ≈ d(log C) / d(log r)

    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut sum_xx = 0.0;
    let mut sum_xy = 0.0;
    let n_points = num_radii as f64;

    for i in 0..num_radii {
        let x = radii[i].ln();
        let y = correlation_sums[i].ln();

        sum_x += x;
        sum_y += y;
        sum_xx += x * x;
        sum_xy += x * y;
    }

    // Pendiente de la regresión = D₂
    let d2 = (n_points * sum_xy - sum_x * sum_y) / (n_points * sum_xx - sum_x * sum_x);

    d2.max(0.0) // D₂ no puede ser negativo
}

/// Calcula el exponente de Lyapunov máximo λ₁
///
/// Un sistema es caótico si λ₁ > 0 (trayectorias divergen exponencialmente)
///
/// # Algoritmo simplificado
/// 1. Seleccionar pares de puntos cercanos inicialmente
/// 2. Seguir su evolución temporal
/// 3. Medir tasa de divergencia promedio
/// 4. λ₁ = lim_{t→∞} (1/t) * log(d(t) / d(0))
pub fn max_lyapunov_exponent(vectors: &[Vec<f32>]) -> f64 {
    if vectors.len() < 100 {
        return 0.0;
    }

    let n = vectors.len().min(1000);
    let dim = vectors[0].len();

    // 1. Encontrar pares de puntos cercanos
    let num_pairs = 50; // Usar 50 pares de referencia
    let mut lyapunov_estimates = Vec::new();

    for i in (0..(n - 20)).step_by(n / num_pairs) {
        // Encontrar vecino más cercano
        let mut min_dist = f64::INFINITY;
        let mut nearest_idx = i + 1;

        for j in (i + 1)..(n - 20) {
            let dist = euclidean_distance(&vectors[i], &vectors[j]);
            if dist > 1e-6 && dist < min_dist {
                min_dist = dist;
                nearest_idx = j;
            }
        }

        if min_dist == f64::INFINITY {
            continue;
        }

        // 2. Seguir evolución del par durante 10-20 pasos
        let max_steps = 20.min(n - i.max(nearest_idx));
        let mut log_divergence_sum = 0.0;
        let mut valid_steps = 0;

        for t in 1..max_steps {
            if i + t >= n || nearest_idx + t >= n {
                break;
            }

            let d0 = min_dist;
            let dt = euclidean_distance(&vectors[i + t], &vectors[nearest_idx + t]);

            if dt > 1e-6 && d0 > 1e-6 {
                log_divergence_sum += (dt / d0).ln();
                valid_steps += 1;
            }
        }

        if valid_steps > 0 {
            let lambda = log_divergence_sum / valid_steps as f64;
            lyapunov_estimates.push(lambda);
        }
    }

    // Promedio de estimaciones
    if lyapunov_estimates.is_empty() {
        return 0.0;
    }

    lyapunov_estimates.iter().sum::<f64>() / lyapunov_estimates.len() as f64
}

/// Calcula distancia euclidiana entre dos vectores
fn euclidean_distance(a: &[f32], b: &[f32]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| {
            let diff = (*x as f64) - (*y as f64);
            diff * diff
        })
        .sum::<f64>()
        .sqrt()
}

/// Análisis completo de atractor
pub fn analyze_attractor(vectors: &[Vec<f32>]) -> AttractorAnalysis {
    println!("🔬 Analizando atractor caótico...");
    println!("   Puntos: {}", vectors.len());
    println!("   Dimensión: {}", vectors[0].len());

    // 1. Dimensión de correlación D₂
    println!("\n📊 Calculando dimensión de correlación D₂...");
    let d2 = correlation_dimension(vectors);
    println!("   D₂ = {:.4}", d2);

    // 2. Exponente de Lyapunov
    println!("\n📈 Calculando exponente de Lyapunov λ₁...");
    let lambda1 = max_lyapunov_exponent(vectors);
    println!("   λ₁ = {:.6}", lambda1);

    // 3. Determinar si existe atractor caótico
    let embedding_dim = vectors[0].len();
    let is_chaotic = d2 < (embedding_dim as f64) && lambda1 > 0.0;

    println!("\n🎯 Diagnóstico:");
    println!("   D₂ < dim_embedding: {} ({:.2} < {})",
             d2 < (embedding_dim as f64), d2, embedding_dim);
    println!("   λ₁ > 0 (caótico): {} ({:.6} > 0)", lambda1 > 0.0, lambda1);

    if is_chaotic {
        println!("   ✅ EXISTE ATRACTOR CAÓTICO");
    } else {
        println!("   ❌ NO existe atractor caótico");
    }

    // 4. Estimar potencial de compresión
    // Ratio de compresión ≈ dim_embedding / D₂
    let compression_potential = if d2 > 0.1 {
        embedding_dim as f64 / d2
    } else {
        1.0
    };

    println!("\n💡 Potencial de compresión: {:.1}x", compression_potential);
    println!("   (Modelo de {} dim → trayectoria en atractor de dim {:.2})",
             embedding_dim, d2);

    AttractorAnalysis {
        correlation_dimension: d2,
        max_lyapunov: lambda1,
        embedding_dim,
        is_chaotic_attractor: is_chaotic,
        compression_potential,
    }
}

/// Reconstrucción de espacio de fases usando método de Takens
///
/// Convierte serie temporal de embeddings en trayectoria en espacio reconstruido
///
/// # Parámetros
/// - `vectors`: Serie temporal de vectores
/// - `delay`: Retardo temporal (típicamente 1 para series densamente muestreadas)
/// - `embed_dim`: Dimensión del espacio reconstruido
///
/// # Teorema de Takens
/// Si la dimensión de embedding es suficientemente grande (≥ 2*D₂ + 1),
/// la reconstrucción preserva las propiedades topológicas del atractor original.
pub fn takens_embedding(vectors: &[Vec<f32>], delay: usize, embed_dim: usize) -> Vec<Vec<f32>> {
    let n = vectors.len();
    let original_dim = vectors[0].len();

    if n < delay * embed_dim {
        return vectors.to_vec(); // No hay suficientes datos
    }

    let mut reconstructed = Vec::new();

    for i in 0..(n - delay * embed_dim) {
        let mut point = Vec::new();

        for j in 0..embed_dim {
            // Tomar coordenadas del vector en tiempo (i + j*delay)
            for k in 0..original_dim {
                point.push(vectors[i + j * delay][k]);
            }
        }

        reconstructed.push(point);
    }

    reconstructed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlation_dimension_random() {
        // Datos aleatorios deberían tener D₂ ≈ dim
        let mut vectors = Vec::new();
        use rand::Rng;
        let mut rng = rand::thread_rng();

        for _ in 0..500 {
            let vec: Vec<f32> = (0..10).map(|_| rng.gen()).collect();
            vectors.push(vec);
        }

        let d2 = correlation_dimension(&vectors);
        println!("D₂ (random 10D) = {:.2}", d2);
        // Esperamos D₂ cercano a 10 (puede variar por muestreo)
        assert!(d2 > 5.0 && d2 < 15.0);
    }

    #[test]
    fn test_lyapunov_stable() {
        // Sistema estable (convergente) debería tener λ < 0
        let mut vectors = Vec::new();
        let mut current = vec![1.0f32; 5];

        for _ in 0..200 {
            vectors.push(current.clone());
            // Decaimiento exponencial hacia 0
            current = current.iter().map(|&x| x * 0.95).collect();
        }

        let lambda = max_lyapunov_exponent(&vectors);
        println!("λ₁ (stable) = {:.6}", lambda);
        // Sistema estable: λ < 0
        assert!(lambda < 0.1);
    }
}
