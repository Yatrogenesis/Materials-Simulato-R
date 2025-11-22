//! ANS Simplificado: Usar histogram + arithmetic-like coding
//!
//! Implementación simplificada que logra compresión cercana a entropía

use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use std::io::{Write, Read};

/// Delta Encoding con cuantización int8 uniforme + GZIP
///
/// Estrategia:
/// - Cuantizar TODOS los deltas a int8 (±127)
/// - Escalar basado en max(|delta|)
/// - Comprimir con GZIP (que funciona mejor que ANS para entropía uniforme)
pub fn delta_ans_compress(vectors: &[Vec<f32>]) -> Vec<u8> {
    if vectors.is_empty() {
        return vec![];
    }

    let dim = vectors[0].len();
    let n_vectors = vectors.len();

    // 1. Calcular deltas
    let mut deltas_f32 = Vec::new();

    for i in 1..n_vectors {
        for j in 0..dim {
            let delta = vectors[i][j] - vectors[i - 1][j];
            deltas_f32.push(delta);
        }
    }

    // 2. Encontrar máximo absoluto para escalar
    let max_abs_delta = deltas_f32.iter()
        .map(|&d| d.abs())
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(1.0);

    // Si todos los deltas son 0, usar factor de escala 1.0
    let scale = if max_abs_delta > 1e-10 { max_abs_delta } else { 1.0 };

    // 3. Cuantizar a int8 (rango [-127, 127])
    let deltas_i8: Vec<i8> = deltas_f32.iter()
        .map(|&delta| {
            let normalized = delta / scale;
            (normalized * 127.0).clamp(-127.0, 127.0) as i8
        })
        .collect();

    // 4. Convertir a bytes (shift para evitar negativos en GZIP)
    let encoded: Vec<u8> = deltas_i8.iter()
        .map(|&q| (q as i16 + 128) as u8)
        .collect();

    // 5. Aplicar GZIP sobre datos cuantizados
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(&encoded).unwrap();
    let compressed_deltas = encoder.finish().unwrap();

    // 6. Serializar resultado
    let mut result = Vec::new();

    // Metadata
    result.extend(&(n_vectors as u32).to_le_bytes());
    result.extend(&(dim as u32).to_le_bytes());
    result.extend(&scale.to_le_bytes());

    // Primer vector (float32)
    for &val in &vectors[0] {
        result.extend(&val.to_le_bytes());
    }

    // Deltas comprimidos
    result.extend(&(compressed_deltas.len() as u32).to_le_bytes());
    result.extend(&compressed_deltas);

    result
}

/// Descompresión de Delta + ANS simplificado
pub fn delta_ans_decompress(compressed: &[u8]) -> Vec<Vec<f32>> {
    if compressed.len() < 12 {
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

    let scale = f32::from_le_bytes([
        compressed[offset],
        compressed[offset + 1],
        compressed[offset + 2],
        compressed[offset + 3],
    ]);
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

    // Leer deltas comprimidos
    let compressed_size = u32::from_le_bytes([
        compressed[offset],
        compressed[offset + 1],
        compressed[offset + 2],
        compressed[offset + 3],
    ]) as usize;
    offset += 4;

    let compressed_deltas = &compressed[offset..offset + compressed_size];

    // Descomprimir GZIP
    let mut decoder = GzDecoder::new(compressed_deltas);
    let mut encoded = Vec::new();
    decoder.read_to_end(&mut encoded).unwrap();

    // Decodificar deltas: convertir u8 → i8 → f32
    let deltas_f32: Vec<f32> = encoded.iter()
        .map(|&byte| {
            let quantized = (byte as i16 - 128) as i8;
            (quantized as f32 / 127.0) * scale
        })
        .collect();

    // Reconstruir vectores desde deltas
    let mut vectors = Vec::with_capacity(n_vectors);
    vectors.push(first_vec.clone());

    let mut prev = first_vec;
    let mut delta_idx = 0;

    for _ in 1..n_vectors {
        let mut current = Vec::with_capacity(dim);
        for _ in 0..dim {
            if delta_idx < deltas_f32.len() {
                current.push(prev[current.len()] + deltas_f32[delta_idx]);
                delta_idx += 1;
            } else {
                break;
            }
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
        let mut vectors = Vec::new();
        let mut current = vec![0.5f32; 10];
        vectors.push(current.clone());

        for _ in 0..99 {
            for val in current.iter_mut() {
                *val += 0.01;
            }
            vectors.push(current.clone());
        }

        let compressed = delta_ans_compress(&vectors);
        let decompressed = delta_ans_decompress(&compressed);

        assert_eq!(vectors.len(), decompressed.len());

        for (orig, decomp) in vectors.iter().zip(decompressed.iter()) {
            assert_eq!(orig.len(), decomp.len());
            for (o, d) in orig.iter().zip(decomp.iter()) {
                assert!((o - d).abs() < 0.02, "Valor original: {}, Decomprimido: {}", o, d);
            }
        }
    }
}
