//! Delta Encoding LOSSLESS con RLE + GZIP
//!
//! Estrategia:
//! 1. Calcular deltas (float32)
//! 2. Aplicar Run-Length Encoding para deltas repetidos
//! 3. Comprimir con GZIP
//! 4. Sin pérdida de información

use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use std::io::{Write, Read};

/// Delta Encoding lossless con RLE + GZIP
pub fn delta_lossless_compress(vectors: &[Vec<f32>]) -> Vec<u8> {
    if vectors.is_empty() {
        return vec![];
    }

    let dim = vectors[0].len();
    let n_vectors = vectors.len();

    // 1. Calcular deltas (float32, sin cuantización)
    let mut deltas_f32 = Vec::new();

    for i in 1..n_vectors {
        for j in 0..dim {
            let delta = vectors[i][j] - vectors[i - 1][j];
            deltas_f32.push(delta);
        }
    }

    // 2. Aplicar Run-Length Encoding simple
    // Si hay deltas repetidos consecutivos, codificar como (count, value)
    let mut rle_encoded = Vec::new();

    if !deltas_f32.is_empty() {
        let mut current_value = deltas_f32[0];
        let mut count = 1u32;

        for &delta in &deltas_f32[1..] {
            if (delta - current_value).abs() < 1e-10 && count < u32::MAX {
                count += 1;
            } else {
                // Escribir (count, value)
                rle_encoded.extend(&count.to_le_bytes());
                rle_encoded.extend(&current_value.to_le_bytes());
                current_value = delta;
                count = 1;
            }
        }

        // Último valor
        rle_encoded.extend(&count.to_le_bytes());
        rle_encoded.extend(&current_value.to_le_bytes());
    }

    // 3. Comprimir con GZIP
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(&rle_encoded).unwrap();
    let compressed_deltas = encoder.finish().unwrap();

    // 4. Serializar resultado
    let mut result = Vec::new();

    // Metadata
    result.extend(&(n_vectors as u32).to_le_bytes());
    result.extend(&(dim as u32).to_le_bytes());

    // Primer vector (float32)
    for &val in &vectors[0] {
        result.extend(&val.to_le_bytes());
    }

    // Deltas comprimidos
    result.extend(&(compressed_deltas.len() as u32).to_le_bytes());
    result.extend(&compressed_deltas);

    result
}

/// Descompresión de Delta lossless
pub fn delta_lossless_decompress(compressed: &[u8]) -> Vec<Vec<f32>> {
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
    let mut rle_encoded = Vec::new();
    decoder.read_to_end(&mut rle_encoded).unwrap();

    // Decodificar RLE
    let mut deltas_f32 = Vec::new();
    let mut i = 0;
    while i + 7 < rle_encoded.len() {
        let count = u32::from_le_bytes([
            rle_encoded[i],
            rle_encoded[i + 1],
            rle_encoded[i + 2],
            rle_encoded[i + 3],
        ]) as usize;
        i += 4;

        let value = f32::from_le_bytes([
            rle_encoded[i],
            rle_encoded[i + 1],
            rle_encoded[i + 2],
            rle_encoded[i + 3],
        ]);
        i += 4;

        // Expandir RLE
        for _ in 0..count {
            deltas_f32.push(value);
        }
    }

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
    fn test_delta_lossless_roundtrip() {
        let mut vectors = Vec::new();
        let mut current = vec![0.5f32; 10];
        vectors.push(current.clone());

        for _ in 0..99 {
            for val in current.iter_mut() {
                *val += 0.01;
            }
            vectors.push(current.clone());
        }

        let compressed = delta_lossless_compress(&vectors);
        let decompressed = delta_lossless_decompress(&compressed);

        assert_eq!(vectors.len(), decompressed.len());

        for (orig, decomp) in vectors.iter().zip(decompressed.iter()) {
            assert_eq!(orig.len(), decomp.len());
            for (o, d) in orig.iter().zip(decomp.iter()) {
                // Exacto (sin pérdida)
                assert_eq!(o, d, "Valor original: {}, Decomprimido: {}", o, d);
            }
        }
    }
}
