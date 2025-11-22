//! Generadores de datasets sintéticos para pruebas de compresión

use rand::Rng;

/// Calcula similitud coseno promedio entre vectores consecutivos
pub fn calculate_consecutive_similarity(vectors: &[Vec<f32>]) -> f64 {
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

fn normalize_vector(v: &mut [f32]) {
    let norm: f32 = v.iter().map(|&x| x * x).sum::<f32>().sqrt();
    if norm > 1e-10 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
}

/// Genera vectores con DRIFT ACUMULATIVO (similitud consecutiva)
pub fn generate_conversational_drift(n: usize, dim: usize, drift_rate: f64) -> Vec<Vec<f32>> {
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
        current = next;
    }

    vectors
}

/// Genera vectores con TEMPORAL SMOOTHING (promedio móvil exponencial)
pub fn generate_temporal_smoothing(n: usize, dim: usize, alpha: f64) -> Vec<Vec<f32>> {
    let mut rng = rand::thread_rng();
    let mut vectors = Vec::new();
    let mut current: Vec<f32> = (0..dim).map(|_| rng.gen::<f32>()).collect();
    normalize_vector(&mut current);
    vectors.push(current.clone());

    for _ in 1..n {
        let noise: Vec<f32> = (0..dim).map(|_| rng.gen::<f32>()).collect();
        let mut next = vec![0.0; dim];

        for i in 0..dim {
            next[i] = current[i] * (alpha as f32) + noise[i] * ((1.0 - alpha) as f32);
        }

        normalize_vector(&mut next);
        vectors.push(next.clone());
        current = next;
    }

    vectors
}

/// Genera vectores CLUSTERIZADOS (cambio de tema cada N vectores)
pub fn generate_clustered_topics(n: usize, dim: usize, cluster_size: usize) -> Vec<Vec<f32>> {
    let mut rng = rand::thread_rng();

    let num_clusters = (n + cluster_size - 1) / cluster_size;
    let mut cluster_centers = Vec::new();

    // Generar centros de clusters
    for _ in 0..num_clusters {
        let mut center: Vec<f32> = (0..dim).map(|_| rng.gen::<f32>()).collect();
        normalize_vector(&mut center);
        cluster_centers.push(center);
    }

    let mut vectors = Vec::new();

    for i in 0..n {
        let cluster_idx = i / cluster_size;
        let center = &cluster_centers[cluster_idx];

        let noise: Vec<f32> = (0..dim).map(|_| rng.gen::<f32>()).collect();
        let mut vector = vec![0.0; dim];

        for j in 0..dim {
            vector[j] = center[j] * 0.98 + noise[j] * 0.02;
        }

        normalize_vector(&mut vector);
        vectors.push(vector);
    }

    vectors
}
