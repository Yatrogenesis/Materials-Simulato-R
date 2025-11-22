//! Herramienta de Diagnóstico: Análisis de Distribución de Deltas
//!
//! Analiza por qué Delta Encoding no alcanza 8x de compresión
//! Investiga: distribución de valores, entropía, compresibilidad teórica

use compression_experiment::*;
use std::collections::HashMap;

fn main() {
    println!("🔬 ANÁLISIS DE CAUSA RAÍZ: Delta Encoding\n");
    println!("Objetivo: Entender por qué no alcanzamos 8x de compresión\n");
    println!("{}", "=".repeat(80));

    let n_vectors = 1000;
    let dim = 768;

    // Generar dataset con ALTA similitud consecutiva
    println!("\n📊 Generando Conversational Drift (similitud consecutiva ≈0.96)...");
    let vectors = generate_conversational_drift(n_vectors, dim, 0.05);

    let consec_sim = calculate_consecutive_similarity(&vectors);
    println!("✅ Similitud consecutiva: {:.4}\n", consec_sim);

    // ============================================================
    // ANÁLISIS 1: Distribución de Deltas Cartesianas
    // ============================================================
    println!("{}", "=".repeat(80));
    println!("📈 ANÁLISIS 1: Deltas Cartesianas");
    println!("{}", "=".repeat(80));

    let mut delta_magnitudes = Vec::new();
    let mut delta_values = Vec::new();

    for i in 1..vectors.len() {
        for j in 0..vectors[i].len() {
            let delta = vectors[i][j] - vectors[i-1][j];
            delta_values.push(delta);
            delta_magnitudes.push(delta.abs());
        }
    }

    // Estadísticas básicas
    let mean_delta = delta_values.iter().sum::<f32>() / delta_values.len() as f32;
    let mean_abs_delta = delta_magnitudes.iter().sum::<f32>() / delta_magnitudes.len() as f32;

    let mut sorted_mags = delta_magnitudes.clone();
    sorted_mags.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median_delta = sorted_mags[sorted_mags.len() / 2];
    let p95_delta = sorted_mags[(sorted_mags.len() as f32 * 0.95) as usize];
    let max_delta = sorted_mags.last().unwrap();

    println!("\n📊 Estadísticas de Deltas:");
    println!("  Media (signed):     {:>10.6}", mean_delta);
    println!("  Media (absoluta):   {:>10.6}", mean_abs_delta);
    println!("  Mediana:            {:>10.6}", median_delta);
    println!("  Percentil 95:       {:>10.6}", p95_delta);
    println!("  Máximo:             {:>10.6}", max_delta);

    // Distribución en buckets
    println!("\n📊 Distribución de |Δ|:");
    let buckets = [0.0, 0.001, 0.01, 0.05, 0.1, 0.5, 1.0, f32::INFINITY];
    let mut counts = vec![0usize; buckets.len()];

    for &mag in &delta_magnitudes {
        for i in 0..buckets.len()-1 {
            if mag >= buckets[i] && mag < buckets[i+1] {
                counts[i] += 1;
                break;
            }
        }
    }

    for i in 0..buckets.len()-1 {
        let pct = 100.0 * counts[i] as f32 / delta_magnitudes.len() as f32;
        let bar = "█".repeat((pct / 2.0) as usize);
        println!("  [{:>6.3}, {:>6.3}): {:>6.2}% {}",
                 buckets[i], buckets[i+1], pct, bar);
    }

    // ============================================================
    // ANÁLISIS 2: Entropía de Deltas (Shannon)
    // ============================================================
    println!("\n{}", "=".repeat(80));
    println!("📈 ANÁLISIS 2: Entropía de Shannon");
    println!("{}", "=".repeat(80));

    // Cuantizar deltas a int8 para calcular entropía
    let delta_int8: Vec<i8> = delta_values.iter()
        .map(|&d| ((d * 127.0).clamp(-128.0, 127.0) as i8))
        .collect();

    let mut histogram: HashMap<i8, usize> = HashMap::new();
    for &val in &delta_int8 {
        *histogram.entry(val).or_insert(0) += 1;
    }

    let total = delta_int8.len() as f64;
    let mut entropy = 0.0;
    for &count in histogram.values() {
        let p = count as f64 / total;
        entropy -= p * p.log2();
    }

    let max_entropy = 8.0; // 8 bits = log2(256)
    let compression_potential = max_entropy / entropy;

    println!("\n📊 Entropía de deltas cuantizados (int8):");
    println!("  Entropía:              {:>10.4} bits/símbolo", entropy);
    println!("  Entropía máxima:       {:>10.4} bits (uniform)", max_entropy);
    println!("  Símbolos únicos:       {:>10} / 256", histogram.len());
    println!("  Potencial compresión:  {:>10.2}x (teórico)", compression_potential);

    // ============================================================
    // ANÁLISIS 3: Compresibilidad Real vs Teórica
    // ============================================================
    println!("\n{}", "=".repeat(80));
    println!("📈 ANÁLISIS 3: Compresibilidad Real vs Teórica");
    println!("{}", "=".repeat(80));

    // Tamaño original
    let original_size = vectors.len() * vectors[0].len() * 4; // float32

    // Tamaño de deltas sin comprimir (float32)
    let delta_uncompressed = (vectors.len() - 1) * dim * 4 + dim * 4; // deltas + primer vector

    // Tamaño de deltas comprimido con GZIP
    let delta_compressed_gzip = {
        let mut data = Vec::new();
        data.extend(vectors[0].iter().flat_map(|&f| f.to_le_bytes()));
        for i in 1..vectors.len() {
            for j in 0..vectors[i].len() {
                let delta = vectors[i][j] - vectors[i-1][j];
                data.extend(&delta.to_le_bytes());
            }
        }
        use flate2::Compression;
        use flate2::write::GzEncoder;
        use std::io::Write;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(&data).unwrap();
        encoder.finish().unwrap().len()
    };

    // Tamaño teórico según entropía
    let theoretical_size = ((entropy / 8.0) * delta_int8.len() as f64) as usize;

    println!("\n📊 Tamaños y Compresión:");
    println!("  Original:                {:>12} bytes", original_size);
    println!("  Deltas sin comprimir:    {:>12} bytes ({:.2}x)",
             delta_uncompressed, original_size as f32 / delta_uncompressed as f32);
    println!("  Deltas + GZIP (real):    {:>12} bytes ({:.2}x)",
             delta_compressed_gzip, original_size as f32 / delta_compressed_gzip as f32);
    println!("  Teórico (entropía):      {:>12} bytes ({:.2}x)",
             theoretical_size, original_size as f32 / theoretical_size as f32);

    let efficiency = (theoretical_size as f32 / delta_compressed_gzip as f32) * 100.0;
    println!("\n  Eficiencia GZIP:         {:>10.2}% del teórico", efficiency);

    // ============================================================
    // ANÁLISIS 4: Deltas Polares
    // ============================================================
    println!("\n{}", "=".repeat(80));
    println!("📈 ANÁLISIS 4: Deltas en Espacio Polar");
    println!("{}", "=".repeat(80));

    fn to_spherical_angles_dbg(vec: &[f32]) -> Vec<f32> {
        let n = vec.len();
        let mut angles = Vec::with_capacity(n - 1);
        let mut r_squared: f64 = vec.iter().map(|&x| (x as f64) * (x as f64)).sum();

        for i in 0..n-2 {
            if r_squared > 1e-10 {
                let r = r_squared.sqrt();
                let cos_theta = (vec[i] as f64) / r;
                let theta = cos_theta.clamp(-1.0, 1.0).acos();
                angles.push(theta as f32);
                r_squared -= (vec[i] as f64) * (vec[i] as f64);
            } else {
                angles.push(0.0);
            }
        }

        if n >= 2 {
            let phi = (vec[n-1] as f64).atan2(vec[n-2] as f64);
            angles.push(phi as f32);
        }

        angles
    }

    let polar_vecs: Vec<Vec<f32>> = vectors.iter()
        .map(|v| to_spherical_angles_dbg(v))
        .collect();

    let mut angular_deltas = Vec::new();
    for i in 1..polar_vecs.len() {
        for j in 0..polar_vecs[i].len() {
            let mut delta = polar_vecs[i][j] - polar_vecs[i-1][j];

            // Normalizar delta a [-π, π]
            while delta > std::f32::consts::PI {
                delta -= 2.0 * std::f32::consts::PI;
            }
            while delta < -std::f32::consts::PI {
                delta += 2.0 * std::f32::consts::PI;
            }

            angular_deltas.push(delta);
        }
    }

    let mean_ang = angular_deltas.iter().map(|x| x.abs()).sum::<f32>() / angular_deltas.len() as f32;
    let mut sorted_ang = angular_deltas.iter().map(|x| x.abs()).collect::<Vec<f32>>();
    sorted_ang.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median_ang = sorted_ang[sorted_ang.len() / 2];
    let p95_ang = sorted_ang[(sorted_ang.len() as f32 * 0.95) as usize];
    let max_ang = sorted_ang.last().unwrap();

    println!("\n📊 Estadísticas de Deltas Angulares:");
    println!("  Media (absoluta):   {:>10.6} rad ({:>6.2}°)", mean_ang, mean_ang.to_degrees());
    println!("  Mediana:            {:>10.6} rad ({:>6.2}°)", median_ang, median_ang.to_degrees());
    println!("  Percentil 95:       {:>10.6} rad ({:>6.2}°)", p95_ang, p95_ang.to_degrees());
    println!("  Máximo:             {:>10.6} rad ({:>6.2}°)", max_ang, max_ang.to_degrees());

    // Distribución angular
    println!("\n📊 Distribución de |Δθ|:");
    let ang_buckets = [0.0, 0.001, 0.01, 0.1, 0.5, 1.0, 2.0, std::f32::consts::PI];
    let mut ang_counts = vec![0usize; ang_buckets.len()];

    for &delta in &angular_deltas {
        let mag = delta.abs();
        for i in 0..ang_buckets.len()-1 {
            if mag >= ang_buckets[i] && mag < ang_buckets[i+1] {
                ang_counts[i] += 1;
                break;
            }
        }
    }

    for i in 0..ang_buckets.len()-1 {
        let pct = 100.0 * ang_counts[i] as f32 / angular_deltas.len() as f32;
        let bar = "█".repeat((pct / 2.0) as usize);
        println!("  [{:>5.3}, {:>5.3}): {:>6.2}% {}",
                 ang_buckets[i], ang_buckets[i+1], pct, bar);
    }

    // ============================================================
    // ANÁLISIS 5: DIAGNÓSTICO FINAL
    // ============================================================
    println!("\n{}", "=".repeat(80));
    println!("🎯 DIAGNÓSTICO FINAL");
    println!("{}", "=".repeat(80));

    println!("\n❓ Por qué Delta Cartesiano solo logra 1.10x:");
    if mean_abs_delta > 0.01 {
        println!("  ❌ Deltas demasiado grandes: media = {:.6}", mean_abs_delta);
        println!("     → Valores no están suficientemente correlacionados");
    }
    if histogram.len() > 200 {
        println!("  ❌ Alta diversidad de deltas: {} símbolos únicos de 256", histogram.len());
        println!("     → GZIP no puede comprimir efectivamente");
    }
    if entropy > 7.0 {
        println!("  ❌ Alta entropía: {:.2} bits (cerca de máximo 8.0)", entropy);
        println!("     → Deltas casi aleatorios, no comprimibles");
    }

    println!("\n❓ Por qué Polar Delta logra 2.6x (mejor pero <8x):");
    if *max_ang > 0.01 {
        println!("  ⚠️  Deltas angulares medianos: {:.3} rad ({:.1}°)",
                 median_ang, median_ang.to_degrees());
        println!("  ⚠️  Deltas angulares máximos: {:.3} rad ({:.1}°)",
                 *max_ang, max_ang.to_degrees());
        println!("     → Esperábamos <0.01 rad para alta compresibilidad");
    }
    if ang_counts.len() >= 2 && ang_counts[ang_counts.len()-2] > angular_deltas.len() / 10 {
        println!("  ⚠️  Muchos deltas grandes (>1 rad): {:.1}%",
                 100.0 * ang_counts[ang_counts.len()-2] as f32 / angular_deltas.len() as f32);
        println!("     → Conversión esférica introduce saltos angulares");
    }

    println!("\n✅ Recomendaciones:");
    println!("  1. Normalizar vectores ANTES de generar datasets");
    println!("  2. Usar cuantización adaptativa (menos bits para deltas pequeños)");
    println!("  3. Considerar transformada decorreladora (KLT) antes de Delta");
    println!("  4. Explorar codificación aritmética en vez de GZIP");

    println!("\n{}", "=".repeat(80));
}
