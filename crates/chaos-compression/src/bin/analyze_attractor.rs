//! Análisis de Atractor Caótico en Embeddings ML
//!
//! **Objetivo**: Determinar si los embeddings viven en un atractor caótico
//! de baja dimensión, lo que permitiría compresión dramática (30-100x).

use compression_experiment::attractor_analysis::*;

// Copiar funciones de generación de datasets desde main.rs
fn generate_conversational_drift(n: usize, dim: usize, drift_rate: f64) -> Vec<Vec<f32>> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut vectors = Vec::new();

    let mut current: Vec<f32> = (0..dim).map(|_| rng.gen::<f32>()).collect();
    normalize_vector(&mut current);
    vectors.push(current.clone());

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

fn generate_temporal_smoothing(n: usize, dim: usize, alpha: f64) -> Vec<Vec<f32>> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut vectors = Vec::new();

    let mut current: Vec<f32> = (0..dim).map(|_| rng.gen::<f32>()).collect();
    normalize_vector(&mut current);
    vectors.push(current.clone());

    for _ in 1..n {
        let noise: Vec<f32> = (0..dim).map(|_| rng.gen::<f32>()).collect();
        let mut next = vec![0.0; dim];

        for i in 0..dim {
            next[i] = current[i] * (alpha as f32) + noise[i] * (1.0 - alpha as f32);
        }

        normalize_vector(&mut next);
        vectors.push(next.clone());
        current = next;
    }

    vectors
}

fn generate_clustered_topics(n: usize, dim: usize, cluster_size: usize) -> Vec<Vec<f32>> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut vectors = Vec::new();

    let num_clusters = (n + cluster_size - 1) / cluster_size;
    let mut cluster_centers = Vec::new();

    for _ in 0..num_clusters {
        let center: Vec<f32> = (0..dim).map(|_| rng.gen::<f32>()).collect();
        cluster_centers.push(center);
    }

    for cluster_idx in 0..num_clusters {
        let center = &cluster_centers[cluster_idx];

        for _ in 0..cluster_size.min(n - vectors.len()) {
            let mut vec = vec![0.0; dim];

            for i in 0..dim {
                vec[i] = center[i] + (rng.gen::<f32>() - 0.5) * 0.1;
            }

            normalize_vector(&mut vec);
            vectors.push(vec);

            if vectors.len() >= n {
                break;
            }
        }
    }

    vectors
}

fn normalize_vector(vec: &mut [f32]) {
    let norm: f32 = vec.iter().map(|&x| x * x).sum::<f32>().sqrt();
    if norm > 1e-10 {
        for x in vec.iter_mut() {
            *x /= norm;
        }
    }
}

fn main() {
    println!("================================================================================");
    println!("🌀 ANÁLISIS DE ATRACTOR CAÓTICO - Embeddings ML");
    println!("================================================================================");
    println!("Autor: Francisco Molina Burgos (ORCID: 0009-0008-6093-8267)");
    println!("Fecha: 2025-11-21");
    println!();
    println!("Objetivo: Determinar si existe un atractor caótico de baja dimensión");
    println!("que permita compresión dramática (30-100x).");
    println!("================================================================================\n");

    // Parámetros
    let n_vectors = 2000; // Más puntos para mejor estimación
    let dim = 768;

    // Probar los 3 datasets más realistas
    let datasets = vec![
        ("Conversational Drift (5%)", generate_conversational_drift(n_vectors, dim, 0.05)),
        ("Temporal Smoothing (α=0.9)", generate_temporal_smoothing(n_vectors, dim, 0.9)),
        ("Clustered Topics (100/cluster)", generate_clustered_topics(n_vectors, dim, 100)),
    ];

    let mut results = Vec::new();

    for (label, vectors) in datasets {
        println!("\n{}", "=".repeat(80));
        println!("📊 Dataset: {}", label);
        println!("{}", "=".repeat(80));

        let analysis = analyze_attractor(&vectors);
        results.push((label, analysis));

        println!();
    }

    // Resumen comparativo
    println!("\n{}", "=".repeat(80));
    println!("📊 TABLA COMPARATIVA - ANÁLISIS DE ATRACTORES");
    println!("{}", "=".repeat(80));
    println!();
    println!("{:<35} {:>10} {:>12} {:>12} {:>20}",
             "Dataset", "D₂", "λ₁", "¿Caótico?", "Compresión (teórica)");
    println!("{}", "-".repeat(95));

    for (label, analysis) in &results {
        let chaotic_marker = if analysis.is_chaotic_attractor { "✅ SÍ" } else { "❌ NO" };

        println!("{:<35} {:>10.4} {:>12.6} {:>12} {:>17.1}x",
                 label,
                 analysis.correlation_dimension,
                 analysis.max_lyapunov,
                 chaotic_marker,
                 analysis.compression_potential);
    }

    println!("\n{}", "=".repeat(80));
    println!("🎯 INTERPRETACIÓN");
    println!("{}", "=".repeat(80));
    println!();

    let has_chaotic = results.iter().any(|(_, a)| a.is_chaotic_attractor);

    if has_chaotic {
        println!("✅ SE DETECTÓ AL MENOS UN ATRACTOR CAÓTICO");
        println!();
        println!("Esto significa que:");
        println!("  1. Los embeddings NO ocupan todo el espacio de 768 dimensiones");
        println!("  2. Viven en una variedad de menor dimensión (D₂ < 768)");
        println!("  3. La dinámica es caótica (λ₁ > 0): trayectorias sensibles a condiciones iniciales");
        println!();
        println!("💡 IMPLICACIONES PARA COMPRESIÓN:");
        println!();

        for (label, analysis) in &results {
            if analysis.is_chaotic_attractor {
                println!("  {} - {}", label, "⭐ CANDIDATO PARA COMPRESIÓN POR ATRACTOR");
                println!("    Dimensión efectiva: {:.2} (vs 768 nominal)", analysis.correlation_dimension);
                println!("    Potencial: {:.1}x compresión", analysis.compression_potential);
                println!("    Estrategia: Modelar como trayectoria en atractor + parámetros del modelo");
                println!();
            }
        }

        println!("📝 PRÓXIMO PASO:");
        println!("  → Implementar compresor basado en modelo de atractor (Lorenz generalizado)");
        println!("  → Codificar embeddings como parámetros de trayectoria en vez de puntos individuales");
    } else {
        println!("❌ NO SE DETECTARON ATRACTORES CAÓTICOS");
        println!();
        println!("Posibles razones:");
        println!("  1. Los embeddings son genuinamente de alta dimensión");
        println!("  2. Se necesitan más puntos para detectar el atractor");
        println!("  3. La dinámica es estocástica, no determinista");
        println!("  4. Los datasets sintéticos no replican la estructura real de embeddings");
        println!();
        println!("📝 RECOMENDACIONES:");
        println!("  1. Probar con embeddings REALES (BERT, GPT, etc.)");
        println!("  2. Aumentar tamaño del dataset (N > 10,000 puntos)");
        println!("  3. Aplicar PCA para reducir ruido antes de análisis");
        println!("  4. Considerar Product Quantization en vez de atractor");
    }

    println!("\n{}", "=".repeat(80));
    println!();

    // Información adicional sobre dimensiones
    println!("📚 CONTEXTO TEÓRICO:");
    println!();
    println!("  Dimensión de Correlación D₂:");
    println!("    • D₂ ≈ dim → espacio completamente ocupado (sin compresión)");
    println!("    • D₂ << dim → estructura de baja dimensión (alta compresibilidad)");
    println!("    • Lorenz: D₂ ≈ 2.05 (atractor 3D)");
    println!("    • Rössler: D₂ ≈ 1.99");
    println!();
    println!("  Exponente de Lyapunov λ₁:");
    println!("    • λ₁ > 0 → caos (divergencia exponencial)");
    println!("    • λ₁ = 0 → punto fijo o ciclo límite");
    println!("    • λ₁ < 0 → convergencia a equilibrio");
    println!();
    println!("  Ratio de compresión = dim_embedding / D₂");
    println!("    • Si dim=768 y D₂=10 → potencial de ~77x");
    println!("    • Si dim=768 y D₂=5 → potencial de ~154x");
    println!();

    println!("================================================================================");
}
