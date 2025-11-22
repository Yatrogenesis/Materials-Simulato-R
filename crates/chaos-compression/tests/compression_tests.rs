use compression_experiment::methods::*;

#[test]
fn test_gzip_roundtrip() {
    let vectors = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]];
    let compressed = gzip_compress(&vectors);
    let decompressed = gzip_decompress(&compressed);
    assert!(!decompressed.is_empty());
}

#[test]
fn test_int8_roundtrip() {
    let vectors = vec![vec![0.5, -0.5, 1.0]];
    let compressed = int8_compress(&vectors);
    let decompressed = int8_decompress(&compressed);
    assert!(!decompressed.is_empty());
}

#[test]
fn test_delta_roundtrip() {
    let vectors = vec![vec![1.0, 2.0], vec![1.1, 2.1]];
    let compressed = delta_compress(&vectors);
    let decompressed = delta_decompress(&compressed);
    assert!(!decompressed.is_empty());
}

#[test]
fn test_zstd_roundtrip() {
    let vectors = vec![vec![1.0; 100]];
    let compressed = zstd_compress(&vectors);
    let decompressed = zstd_decompress(&compressed);
    assert!(!decompressed.is_empty());
}

#[test]
fn test_compression_ratio() {
    let vectors: Vec<Vec<f32>> = vec![vec![1.0; 768]; 100];
    let compressed = gzip_compress(&vectors);
    let original_size = 100 * 768 * 4;
    let ratio = original_size as f64 / compressed.len() as f64;
    assert!(ratio > 1.0);
}
