//! Experimento de Compresión para Vectores ML
//!
//! **Autor**: Francisco Molina Burgos
//! **ORCID**: 0009-0008-6093-8267
//! **Fecha**: 2025-11-21

mod methods;

use methods::*;
use ndarray::{Array1, Array2};
use serde::{Serialize, Deserialize};
use std::time::Instant;

#[derive(Debug, Serialize, Deserialize)]
struct ExperimentResults {
    method: String,
    compression_ratio: f64,
    compression_time_ms: f64,
    decompression_time_ms: f64,
    accuracy_loss: f64,
}

fn main() {
    println!("🔬 Experimento de Compresión de Vectores - CORREGIDO");
    println!("Autor: Francisco Molina Burgos (ORCID: 0009-0008-6093-8267)");
    println!("Fecha: 2025-11-21");
    println!("Versión: 2.0 - Con similitud consecutiva\n");

    let n_vectors = 1000;
    let dim = 768;

    // Probar 4 TIPOS DE DATASETS
    let datasets: Vec<(&str, Vec<Vec<f32>>)> = vec![
        ("Random Similar (baseline)", generate_similar_vectors(n_vectors, dim, 0.8)),
        ("Conversational Drift ⭐ (drift 5%)", generate_conversational_drift(n_vectors, dim, 0.05)),
        ("Temporal Smoothing (alpha 0.9)", generate_temporal_smoothing(n_vectors, dim, 0.9)),
        ("Clustered Topics (100 per cluster)", generate_clustered_topics(n_vectors, dim, 100)),
    ];

    let mut all_results = Vec::new();

    for (label, vectors) in datasets {
        println!("\n{}", "=".repeat(70));
        println!("📊 Testing: {}", label);
        println!("{}", "=".repeat(70));

        // MÉTRICA CRÍTICA: Similitud Consecutiva
        let consec_sim = calculate_consecutive_similarity(&vectors);
        println!("\n🔑 Similitud Consecutiva: {:.4}", consec_sim);

        if consec_sim >= 0.90 {
            println!("   ✅ EXCELENTE - Delta Encoding DEBE funcionar bien (≥8x esperado)");
        } else if consec_sim >= 0.70 {
            println!("   ⚠️  MEDIA - Delta Encoding puede funcionar parcialmente");
        } else {
            println!("   ❌ BAJA - Delta Encoding NO funcionará (método incorrecto)");
        }
        println!();

        let mut results = Vec::new();

        // GZIP Baseline
        println!("Testing GZIP Baseline...");
        results.push(test_method("GZIP", &vectors, gzip_compress, gzip_decompress));

        // Int8 Quantization
        println!("Testing Int8 Quantization...");
        results.push(test_method("Int8+GZIP", &vectors, int8_compress, int8_decompress));

        // Delta Encoding
        println!("Testing Delta Encoding...");
        results.push(test_method("Delta+GZIP", &vectors, delta_compress, delta_decompress));

        // Zstd
        println!("Testing Zstd...");
        results.push(test_method("Zstd", &vectors, zstd_compress, zstd_decompress));

        // Polar Delta ⭐ NUEVO
        println!("Testing Polar Delta Encoding...");
        results.push(test_method("PolarDelta+GZIP", &vectors, polar_delta_compress, polar_delta_decompress));

        // Delta + ANS ⭐⭐⭐ ESPERADO 15x
        println!("Testing Delta + ANS...");
        results.push(test_method("Delta+ANS", &vectors, delta_ans_compress, delta_ans_decompress));

        // Delta Lossless (RLE + GZIP) ⭐⭐⭐⭐ ESPERADO 10-15x SIN pérdida
        println!("Testing Delta Lossless (RLE+GZIP)...");
        results.push(test_method("Delta+RLE+GZIP", &vectors, delta_lossless_compress, delta_lossless_decompress));

        // Attractor Compression (PCA + Delta) ⭐⭐⭐⭐⭐ ESPERADO 100-1000x
        println!("Testing Attractor Compression (PCA+Delta)...");
        results.push(test_method("Attractor(PCA-10)", &vectors, attractor_compress, attractor_decompress));

        println!("\n📊 Resultados:");
        for r in &results {
            let marker = if r.method.contains("Delta") && r.compression_ratio >= 8.0 {
                " ⭐"
            } else if r.method.contains("Delta") && consec_sim >= 0.90 && r.compression_ratio < 8.0 {
                " ⚠️ (esperaba ≥8x)"
            } else {
                ""
            };

            println!("  {:<15}: ratio={:>5.2}x, comp={:>6.2}ms, decomp={:>5.2}ms, loss={:>6.4}%{}",
                     r.method, r.compression_ratio, r.compression_time_ms,
                     r.decompression_time_ms, r.accuracy_loss, marker);
        }

        // Validar hipótesis
        println!("\n🔬 Validación de Hipótesis:");
        let delta_result = results.iter().find(|r| r.method == "Delta+GZIP").unwrap();
        let polar_result = results.iter().find(|r| r.method.contains("PolarDelta")).unwrap();

        // Validación Delta cartesiano
        if consec_sim >= 0.90 && delta_result.compression_ratio >= 8.0 {
            println!("   ✅ Delta Cartesiano: {:.2}x con similitud {:.4}",
                     delta_result.compression_ratio, consec_sim);
        } else if consec_sim >= 0.90 {
            println!("   ❌ Delta Cartesiano: solo {:.2}x (esperaba ≥8x) con similitud {:.4}",
                     delta_result.compression_ratio, consec_sim);
        }

        // Validación Polar Delta ⭐
        if consec_sim >= 0.90 && polar_result.compression_ratio >= 8.0 {
            println!("   ✅ POLAR DELTA VALIDADO: {:.2}x con similitud {:.4} 🎉",
                     polar_result.compression_ratio, consec_sim);
        } else if consec_sim >= 0.90 {
            println!("   ⚠️  Polar Delta: {:.2}x (esperaba ≥8x) con similitud {:.4}",
                     polar_result.compression_ratio, consec_sim);
        }

        // Guardar con label y similitud consecutiva
        all_results.push((label.to_string(), consec_sim, results));
    }

    // Guardar todos los resultados
    let json = serde_json::to_string_pretty(&all_results).unwrap();
    std::fs::write("results/results_all_similarities.json", json).unwrap();

    // Imprimir tabla comparativa
    print_comparison_table(&all_results);
}

fn test_method<C, D>(
    name: &str,
    vectors: &[Vec<f32>],
    compress_fn: C,
    decompress_fn: D,
) -> ExperimentResults
where
    C: Fn(&[Vec<f32>]) -> Vec<u8>,
    D: Fn(&[u8]) -> Vec<Vec<f32>>,
{
    let original_size = vectors.len() * vectors[0].len() * 4;

    let start = Instant::now();
    let compressed = compress_fn(vectors);
    let comp_time = start.elapsed().as_secs_f64() * 1000.0;

    let start = Instant::now();
    let decompressed = decompress_fn(&compressed);
    let decomp_time = start.elapsed().as_secs_f64() * 1000.0;

    let ratio = original_size as f64 / compressed.len() as f64;
    let loss = calculate_accuracy_loss(vectors, &decompressed);

    ExperimentResults {
        method: name.to_string(),
        compression_ratio: ratio,
        compression_time_ms: comp_time,
        decompression_time_ms: decomp_time,
        accuracy_loss: loss,
    }
}

fn generate_similar_vectors(n: usize, dim: usize, similarity: f64) -> Vec<Vec<f32>> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let base: Vec<f32> = (0..dim).map(|_| rng.gen::<f32>()).collect();

    (0..n).map(|_| {
        base.iter().map(|&v| {
            if rng.gen::<f64>() < similarity {
                v + rng.gen::<f32>() * 0.1
            } else {
                rng.gen::<f32>()
            }
        }).collect()
    }).collect()
}

/// NUEVO: Genera vectores con DRIFT ACUMULATIVO (similitud consecutiva)
/// Esto es lo que Delta Encoding necesita para funcionar bien
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
        current = next;
    }

    vectors
}

/// Genera vectores con TEMPORAL SMOOTHING (promedio móvil exponencial)
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
            next[i] = current[i] * (alpha as f32) + noise[i] * ((1.0 - alpha) as f32);
        }

        normalize_vector(&mut next);
        vectors.push(next.clone());
        current = next;
    }

    vectors
}

/// Genera vectores CLUSTERIZADOS (cambio de tema cada N vectores)
fn generate_clustered_topics(n: usize, dim: usize, cluster_size: usize) -> Vec<Vec<f32>> {
    use rand::Rng;
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

fn normalize_vector(v: &mut [f32]) {
    let norm: f32 = v.iter().map(|&x| x * x).sum::<f32>().sqrt();
    if norm > 1e-10 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
}

/// MÉTRICA CRÍTICA: Similitud Consecutiva (cosine similarity promedio)
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

fn calculate_accuracy_loss(original: &[Vec<f32>], decompressed: &[Vec<f32>]) -> f64 {
    let mut total_error = 0.0_f64;
    let mut count = 0;

    for (orig, decomp) in original.iter().zip(decompressed.iter()) {
        for (&o, &d) in orig.iter().zip(decomp.iter()) {
            total_error += ((o - d) / o.max(1e-10)).abs() as f64;
            count += 1;
        }
    }

    (total_error / count as f64) * 100.0
}

fn print_comparison_table(all_results: &[(String, f64, Vec<ExperimentResults>)]) {
    println!("\n\n{}", "=".repeat(90));
    println!("📊 TABLA COMPARATIVA FINAL");
    println!("{}\n", "=".repeat(90));

    println!("{:<30} {:>12} {:>12} {:>12} {:>12} {:>12} {:>15} {:>12}",
             "Dataset", "Consec.Sim", "GZIP", "Int8+GZIP", "Delta+GZIP", "Zstd", "PolarDelta", "Delta+ANS");
    println!("{:-<117}", "");

    for (label, consec_sim, results) in all_results {
        print!("{:<30} {:>11.4} ", label, consec_sim);
        for r in results {
            print!(" {:>11.2}x", r.compression_ratio);
        }
        println!();
    }

    println!("\n{}", "=".repeat(90));
    println!("📊 Pérdida de Accuracy por Dataset");
    println!("{}\n", "=".repeat(90));

    println!("{:<30} {:>12} {:>12} {:>12} {:>12} {:>15} {:>12}",
             "Dataset", "GZIP", "Int8+GZIP", "Delta+GZIP", "Zstd", "PolarDelta", "Delta+ANS");
    println!("{:-<105}", "");

    for (label, _consec_sim, results) in all_results {
        print!("{:<30}", label);
        for r in results {
            print!(" {:>11.4}%", r.accuracy_loss);
        }
        println!();
    }

    println!("\n{}", "=".repeat(90));
    println!("🏆 VALIDACIÓN DE HIPÓTESIS Y CONCLUSIONES");
    println!("{}\n", "=".repeat(90));

    let mut delta_validated_count = 0;
    let mut delta_failed_count = 0;

    for (label, consec_sim, results) in all_results {
        let delta = results.iter().find(|r| r.method.contains("Delta")).unwrap();

        println!("\n{}", label);
        println!("  Similitud Consecutiva: {:.4}", consec_sim);
        println!("  Delta+GZIP: {:.2}x", delta.compression_ratio);

        if *consec_sim >= 0.90 {
            if delta.compression_ratio >= 8.0 {
                println!("  ✅ HIPÓTESIS VALIDADA: Delta ≥8x con alta similitud");
                delta_validated_count += 1;
            } else {
                println!("  ❌ HIPÓTESIS REFUTADA: Delta <8x con alta similitud (esperaba ≥8x)");
                delta_failed_count += 1;
            }
        } else {
            if delta.compression_ratio < 2.0 {
                println!("  ✅ CONTROL CORRECTO: Delta falló con baja similitud (como se esperaba)");
            } else {
                println!("  ⚠️  Delta funcionó mejor de lo esperado con baja similitud");
            }
        }

        // Mejor método para este dataset
        let best = results.iter()
            .max_by(|a, b| a.compression_ratio.partial_cmp(&b.compression_ratio).unwrap())
            .unwrap();

        println!("  🎯 Mejor método: {} ({:.2}x, {:.4}% loss)",
                 best.method, best.compression_ratio, best.accuracy_loss);
    }

    println!("\n{}", "=".repeat(90));
    println!("📝 RESUMEN FINAL");
    println!("{}", "=".repeat(90));
    println!();
    println!("Datasets con similitud consecutiva ≥0.90 donde Delta validó (≥8x): {}", delta_validated_count);
    println!("Datasets con similitud consecutiva ≥0.90 donde Delta falló (<8x): {}", delta_failed_count);
    println!();

    if delta_validated_count > 0 {
        println!("✅ CONCLUSIÓN: Delta Encoding ES EFECTIVO para datos con alta similitud consecutiva.");
        println!("   Recomendación: Implementar en Lirasion para memoria conversacional.");
    } else {
        println!("❌ CONCLUSIÓN: Implementación actual de Delta Encoding NO funciona como esperado.");
        println!("   Revisar algoritmo o considerar alternativas (PCA+Delta, etc.).");
    }
    println!();
}
