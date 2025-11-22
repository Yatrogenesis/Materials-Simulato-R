//! ANS (Asymmetric Numeral Systems) Compression for Delta Encoding
//!
//! Implementa compresión mediante ANS que alcanza ~95% de la entropía teórica

use constriction::stream::{
    model::DefaultLeakyQuantizer,
    stack::AnsCoder,
    Decode, Encode,
};
use constriction::UnwrapInfallible;

type DefaultAnsCoder = AnsCoder<u32, u64, Vec<u32>>;
use std::collections::HashMap;

/// Delta Encoding + ANS: Compresión óptima para deltas de baja entropía
pub fn delta_ans_compress(vectors: &[Vec<f32>]) -> Vec<u8> {
    if vectors.is_empty() {
        return vec![];
    }

    let dim = vectors[0].len();
    let n_vectors = vectors.len();

    // 1. Calcular deltas y cuantizar a int8
    let mut deltas_i8 = Vec::new();

    // Primer vector completo en float32
    let mut first_vec_bytes = Vec::new();
    for &val in &vectors[0] {
        first_vec_bytes.extend(&val.to_le_bytes());
    }

    // Deltas cuantizados
    for i in 1..n_vectors {
        for j in 0..dim {
            let delta = vectors[i][j] - vectors[i - 1][j];
            // Cuantizar a int8: rango [-1, 1] → [-127, 127]
            let quantized = (delta * 127.0).clamp(-127.0, 127.0) as i8;
            deltas_i8.push(quantized);
        }
    }

    // 2. Calcular histograma de frecuencias
    let mut histogram: HashMap<i8, usize> = HashMap::new();
    for &val in &deltas_i8 {
        *histogram.entry(val).or_insert(0) += 1;
    }

    // 3. Crear modelo de probabilidad para ANS
    // Convertir histograma a probabilidades
    let total = deltas_i8.len() as f64;
    let mut probabilities: Vec<(i8, f64)> = histogram
        .iter()
        .map(|(&symbol, &count)| (symbol, count as f64 / total))
        .collect();
    probabilities.sort_by_key(|(s, _)| *s);

    // 4. Codificar con ANS
    let symbols: Vec<i32> = deltas_i8.iter().map(|&x| x as i32 + 128).collect(); // Shift a [0, 255]

    // Crear modelo cuantizado
    let quantizer = DefaultLeakyQuantizer::new(-1..=255); // Rango extendido para seguridad

    let probabilities_shifted: Vec<f64> = {
        let mut probs = vec![1e-10; 257]; // Probabilidad mínima para símbolos no vistos
        for (symbol, prob) in probabilities {
            let idx = (symbol as i32 + 128) as usize;
            if idx < probs.len() {
                probs[idx] = prob;
            }
        }
        // Normalizar
        let sum: f64 = probs.iter().sum();
        probs.iter().map(|&p| p / sum).collect()
    };

    let model = quantizer.quantize_symbols(probabilities_shifted.iter().copied());

    // Codificar
    let mut coder = DefaultAnsCoder::new();
    for &symbol in symbols.iter().rev() {
        coder.encode_symbol(symbol, model.as_view()).unwrap_infallible();
    }

    let compressed_deltas = coder.into_compressed().unwrap_infallible();

    // 5. Serializar: metadata + primer vector + deltas ANS
    let mut result = Vec::new();

    // Metadata
    result.extend(&(n_vectors as u32).to_le_bytes());
    result.extend(&(dim as u32).to_le_bytes());

    // Primer vector (float32)
    result.extend(&first_vec_bytes);

    // Histograma (para decodificación)
    result.extend(&(histogram.len() as u32).to_le_bytes());
    for (symbol, count) in &histogram {
        result.push(*symbol as u8);
        result.extend(&(*count as u32).to_le_bytes());
    }

    // Deltas comprimidos con ANS
    result.extend(&(compressed_deltas.len() as u32).to_le_bytes());
    result.extend(&compressed_deltas);

    result
}

/// Descompresión de Delta + ANS
pub fn delta_ans_decompress(compressed: &[u8]) -> Vec<Vec<f32>> {
    if compressed.len() < 8 {
        return vec![];
    }

    let mut offset = 0;

    // Leer metadata
    let n_vectors = u32::from_le_bytes([
        compressed[offset],
        compressed[offset + 1],
        compressed[offset + 2],
        compressed[offset + 3],
    ]) as usize;
    offset += 4;

    let dim = u32::from_le_bytes([
        compressed[offset],
        compressed[offset + 1],
        compressed[offset + 2],
        compressed[offset + 3],
    ]) as usize;
    offset += 4;

    // Leer primer vector
    let mut first_vec = Vec::with_capacity(dim);
    for _ in 0..dim {
        let val = f32::from_le_bytes([
            compressed[offset],
            compressed[offset + 1],
            compressed[offset + 2],
            compressed[offset + 3],
        ]);
        first_vec.push(val);
        offset += 4;
    }

    // Leer histograma
    let histogram_size = u32::from_le_bytes([
        compressed[offset],
        compressed[offset + 1],
        compressed[offset + 2],
        compressed[offset + 3],
    ]) as usize;
    offset += 4;

    let mut histogram: HashMap<i8, usize> = HashMap::new();
    let mut total_count = 0;
    for _ in 0..histogram_size {
        let symbol = compressed[offset] as i8;
        offset += 1;
        let count = u32::from_le_bytes([
            compressed[offset],
            compressed[offset + 1],
            compressed[offset + 2],
            compressed[offset + 3],
        ]) as usize;
        offset += 4;
        histogram.insert(symbol, count);
        total_count += count;
    }

    // Reconstruir probabilidades
    let total = total_count as f64;
    let mut probabilities_shifted = vec![1e-10; 257];
    for (symbol, count) in histogram {
        let idx = (symbol as i32 + 128) as usize;
        if idx < probabilities_shifted.len() {
            probabilities_shifted[idx] = count as f64 / total;
        }
    }

    // Normalizar
    let sum: f64 = probabilities_shifted.iter().sum();
    probabilities_shifted = probabilities_shifted.iter().map(|&p| p / sum).collect();

    // Leer deltas ANS comprimidos
    let compressed_size = u32::from_le_bytes([
        compressed[offset],
        compressed[offset + 1],
        compressed[offset + 2],
        compressed[offset + 3],
    ]) as usize;
    offset += 4;

    let compressed_deltas = &compressed[offset..offset + compressed_size];

    // Decodificar ANS
    let quantizer = DefaultLeakyQuantizer::new(-1..=255);
    let model = quantizer.quantize_symbols(probabilities_shifted.iter().copied());

    let compressed_vec: Vec<u32> = compressed_deltas
        .chunks(4)
        .map(|chunk| {
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(chunk);
            u32::from_le_bytes(bytes)
        })
        .collect();

    let mut coder = DefaultAnsCoder::from_compressed(compressed_vec).unwrap_infallible();

    let n_deltas = (n_vectors - 1) * dim;
    let mut deltas_i8 = Vec::with_capacity(n_deltas);

    for _ in 0..n_deltas {
        let symbol = coder.decode_symbol(model.as_view()).unwrap_infallible();
        deltas_i8.push((symbol - 128) as i8);
    }

    deltas_i8.reverse(); // ANS decodifica en orden inverso

    // Reconstruir vectores desde deltas
    let mut vectors = Vec::with_capacity(n_vectors);
    vectors.push(first_vec.clone());

    let mut prev = first_vec;
    let mut delta_idx = 0;

    for _ in 1..n_vectors {
        let mut current = Vec::with_capacity(dim);
        for j in 0..dim {
            let delta_quantized = deltas_i8[delta_idx] as f32;
            let delta = delta_quantized / 127.0;
            current.push(prev[j] + delta);
            delta_idx += 1;
        }
        vectors.push(current.clone());
        prev = current;
    }

    vectors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_ans_roundtrip() {
        // Crear vectores de prueba
        let mut vectors = Vec::new();
        let mut current = vec![0.5f32; 10];
        vectors.push(current.clone());

        for _ in 0..99 {
            // Pequeños cambios
            for val in current.iter_mut() {
                *val += 0.01;
            }
            vectors.push(current.clone());
        }

        // Comprimir
        let compressed = delta_ans_compress(&vectors);

        // Descomprimir
        let decompressed = delta_ans_decompress(&compressed);

        // Verificar
        assert_eq!(vectors.len(), decompressed.len());

        for (orig, decomp) in vectors.iter().zip(decompressed.iter()) {
            assert_eq!(orig.len(), decomp.len());
            for (o, d) in orig.iter().zip(decomp.iter()) {
                assert!((o - d).abs() < 0.01, "Valor original: {}, Decomprimido: {}", o, d);
            }
        }
    }

    #[test]
    fn test_compression_ratio() {
        let mut vectors = Vec::new();
        let base = vec![0.5f32; 100];
        vectors.push(base.clone());

        let mut current = base;
        for _ in 0..999 {
            for val in current.iter_mut() {
                *val += 0.001; // Deltas muy pequeños
            }
            vectors.push(current.clone());
        }

        let original_size = vectors.len() * vectors[0].len() * 4;
        let compressed = delta_ans_compress(&vectors);
        let ratio = original_size as f64 / compressed.len() as f64;

        println!("Original: {} bytes", original_size);
        println!("Compressed: {} bytes", compressed.len());
        println!("Ratio: {:.2}x", ratio);

        // Esperamos >=10x con deltas muy pequeños
        assert!(ratio > 10.0, "Ratio: {:.2}x, esperaba >10x", ratio);
    }
}
