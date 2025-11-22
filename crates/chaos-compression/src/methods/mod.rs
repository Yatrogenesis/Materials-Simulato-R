use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use std::io::{Write, Read};

pub mod ans_simple;
pub use ans_simple::{delta_ans_compress, delta_ans_decompress};

pub mod delta_lossless;
pub use delta_lossless::{delta_lossless_compress, delta_lossless_decompress};

pub mod attractor_compression;
pub use attractor_compression::{attractor_compress, attractor_decompress};

pub fn gzip_compress(vectors: &[Vec<f32>]) -> Vec<u8> {
    let bytes: Vec<u8> = vectors.iter()
        .flat_map(|v| v.iter().flat_map(|&f| f.to_le_bytes()))
        .collect();

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&bytes).unwrap();
    encoder.finish().unwrap()
}

pub fn gzip_decompress(compressed: &[u8]) -> Vec<Vec<f32>> {
    let mut decoder = GzDecoder::new(compressed);
    let mut bytes = Vec::new();
    decoder.read_to_end(&mut bytes).unwrap();

    let floats: Vec<f32> = bytes.chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();

    vec![floats] // Simplificado
}

pub fn int8_compress(vectors: &[Vec<f32>]) -> Vec<u8> {
    // Cuantizar a int8
    let bytes: Vec<u8> = vectors.iter()
        .flat_map(|v| v.iter().map(|&f| ((f * 127.0).clamp(-128.0, 127.0) as i8) as u8))
        .collect();

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&bytes).unwrap();
    encoder.finish().unwrap()
}

pub fn int8_decompress(compressed: &[u8]) -> Vec<Vec<f32>> {
    let mut decoder = GzDecoder::new(compressed);
    let mut bytes = Vec::new();
    decoder.read_to_end(&mut bytes).unwrap();

    let floats: Vec<f32> = bytes.iter()
        .map(|&b| (b as i8) as f32 / 127.0)
        .collect();

    vec![floats]
}

pub fn delta_compress(vectors: &[Vec<f32>]) -> Vec<u8> {
    if vectors.is_empty() {
        return vec![];
    }

    let mut deltas = Vec::new();

    // Primera fila completa
    deltas.extend(vectors[0].iter().flat_map(|&f| f.to_le_bytes()));

    // Deltas del resto
    for i in 1..vectors.len() {
        for j in 0..vectors[i].len() {
            let delta = vectors[i][j] - vectors[i-1][j];
            deltas.extend(&delta.to_le_bytes());
        }
    }

    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(&deltas).unwrap();
    encoder.finish().unwrap()
}

pub fn delta_decompress(compressed: &[u8]) -> Vec<Vec<f32>> {
    let mut decoder = GzDecoder::new(compressed);
    let mut bytes = Vec::new();
    decoder.read_to_end(&mut bytes).unwrap();

    let floats: Vec<f32> = bytes.chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();

    vec![floats]
}

pub fn zstd_compress(vectors: &[Vec<f32>]) -> Vec<u8> {
    let bytes: Vec<u8> = vectors.iter()
        .flat_map(|v| v.iter().flat_map(|&f| f.to_le_bytes()))
        .collect();

    zstd::encode_all(&bytes[..], 3).unwrap()
}

pub fn zstd_decompress(compressed: &[u8]) -> Vec<Vec<f32>> {
    let bytes = zstd::decode_all(compressed).unwrap();

    let floats: Vec<f32> = bytes.chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();

    vec![floats]
}

/// Convierte vector cartesiano a ángulos esféricos (hyperspherical coordinates)
/// Para un vector n-dimensional, genera n-1 ángulos
fn to_spherical_angles(vec: &[f32]) -> Vec<f32> {
    let n = vec.len();
    let mut angles = Vec::with_capacity(n - 1);

    // Calcular magnitud total
    let mut r_squared: f64 = vec.iter().map(|&x| (x as f64) * (x as f64)).sum();

    // Primeros n-2 ángulos (coordenadas polares generalizadas)
    for i in 0..n-2 {
        if r_squared > 1e-10 {
            let r = r_squared.sqrt();
            let cos_theta = (vec[i] as f64) / r;
            let theta = cos_theta.clamp(-1.0, 1.0).acos();
            angles.push(theta as f32);

            // Actualizar r² para siguiente iteración
            r_squared -= (vec[i] as f64) * (vec[i] as f64);
        } else {
            angles.push(0.0);
        }
    }

    // Último ángulo (azimuthal en plano final)
    if n >= 2 {
        let phi = (vec[n-1] as f64).atan2(vec[n-2] as f64);
        angles.push(phi as f32);
    }

    angles
}

/// Convierte ángulos esféricos de vuelta a vector cartesiano
fn from_spherical_angles(angles: &[f32], magnitude: f32) -> Vec<f32> {
    let n = angles.len() + 1; // n-1 ángulos → n dimensiones
    let mut vec = vec![0.0f32; n];

    let mut r = magnitude as f64;

    // Reconstruir usando producto de senos
    for i in 0..n-2 {
        let theta = angles[i] as f64;
        vec[i] = (r * theta.cos()) as f32;
        r *= theta.sin();
    }

    // Últimas dos coordenadas del plano final
    if n >= 2 {
        let phi = angles[n-2] as f64;
        vec[n-2] = (r * phi.cos()) as f32;
        vec[n-1] = (r * phi.sin()) as f32;
    }

    vec
}

/// Polar Delta Encoding: convierte a coordenadas esféricas y codifica deltas angulares
pub fn polar_delta_compress(vectors: &[Vec<f32>]) -> Vec<u8> {
    if vectors.is_empty() {
        return vec![];
    }

    let dim = vectors[0].len();
    let n_vectors = vectors.len();

    // Convertir todos los vectores a representación polar
    let polar_vecs: Vec<(f32, Vec<f32>)> = vectors.iter()
        .map(|v| {
            let magnitude = v.iter().map(|&x| x * x).sum::<f32>().sqrt();
            let angles = to_spherical_angles(v);
            (magnitude, angles)
        })
        .collect();

    let mut data = Vec::new();

    // Metadata: número de vectores y dimensiones
    data.extend(&(n_vectors as u32).to_le_bytes());
    data.extend(&(dim as u32).to_le_bytes());

    // Primer vector completo (magnitud + ángulos en float32)
    data.extend(&polar_vecs[0].0.to_le_bytes());
    for &angle in &polar_vecs[0].1 {
        data.extend(&angle.to_le_bytes());
    }

    // Deltas para vectores restantes
    for i in 1..n_vectors {
        // Delta de magnitud (float32)
        let mag_delta = polar_vecs[i].0 - polar_vecs[i-1].0;
        data.extend(&mag_delta.to_le_bytes());

        // Deltas angulares cuantizados a int16 (±π rad → ±32767)
        for j in 0..polar_vecs[i].1.len() {
            let angle_delta = polar_vecs[i].1[j] - polar_vecs[i-1].1[j];
            // Escalar: ±π → ±32767
            let quantized = (angle_delta * (32767.0 / std::f32::consts::PI))
                .clamp(-32768.0, 32767.0) as i16;
            data.extend(&quantized.to_le_bytes());
        }
    }

    // Comprimir con GZIP
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(&data).unwrap();
    encoder.finish().unwrap()
}

/// Descompresión de Polar Delta Encoding
pub fn polar_delta_decompress(compressed: &[u8]) -> Vec<Vec<f32>> {
    let mut decoder = GzDecoder::new(compressed);
    let mut data = Vec::new();
    decoder.read_to_end(&mut data).unwrap();

    if data.len() < 8 {
        return vec![];
    }

    // Leer metadata
    let n_vectors = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let dim = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
    let n_angles = dim - 1;

    let mut vectors = Vec::with_capacity(n_vectors);
    let mut offset = 8;

    // Leer primer vector completo
    let first_magnitude = f32::from_le_bytes([
        data[offset], data[offset+1], data[offset+2], data[offset+3]
    ]);
    offset += 4;

    let mut first_angles = Vec::with_capacity(n_angles);
    for _ in 0..n_angles {
        let angle = f32::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3]
        ]);
        first_angles.push(angle);
        offset += 4;
    }

    vectors.push(from_spherical_angles(&first_angles, first_magnitude));

    // Reconstruir vectores desde deltas
    let mut prev_magnitude = first_magnitude;
    let mut prev_angles = first_angles;

    for _ in 1..n_vectors {
        // Leer delta de magnitud
        let mag_delta = f32::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3]
        ]);
        offset += 4;

        let current_magnitude = prev_magnitude + mag_delta;

        // Leer deltas angulares y desquantizar
        let mut current_angles = Vec::with_capacity(n_angles);
        for j in 0..n_angles {
            let quantized = i16::from_le_bytes([data[offset], data[offset+1]]);
            offset += 2;

            // Desescalar: ±32767 → ±π
            let angle_delta = (quantized as f32) * (std::f32::consts::PI / 32767.0);
            current_angles.push(prev_angles[j] + angle_delta);
        }

        vectors.push(from_spherical_angles(&current_angles, current_magnitude));

        prev_magnitude = current_magnitude;
        prev_angles = current_angles;
    }

    vectors
}
