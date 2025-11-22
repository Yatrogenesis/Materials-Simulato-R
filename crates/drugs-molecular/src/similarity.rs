//! Similarity metrics for molecular comparison
//!
//! Target: 100x faster than RDKit
//! Example: 1M Tanimoto comparisons in 0.6s (vs 60s in Python)

use crate::Fingerprint;

/// Tanimoto (Jaccard) similarity coefficient
/// Target: <0.001μs per comparison
pub fn tanimoto_similarity(fp1: &Fingerprint, fp2: &Fingerprint) -> f64 {
    if fp1.size != fp2.size {
        return 0.0;
    }

    let mut intersection = 0;
    let mut union = 0;

    for i in 0..fp1.size {
        let b1 = fp1.get_bit(i);
        let b2 = fp2.get_bit(i);

        if b1 && b2 {
            intersection += 1;
        }
        if b1 || b2 {
            union += 1;
        }
    }

    if union == 0 {
        return 0.0;
    }

    intersection as f64 / union as f64
}

/// Dice similarity coefficient
pub fn dice_similarity(fp1: &Fingerprint, fp2: &Fingerprint) -> f64 {
    if fp1.size != fp2.size {
        return 0.0;
    }

    let mut intersection = 0;
    let count1 = fp1.count_on_bits();
    let count2 = fp2.count_on_bits();

    for i in 0..fp1.size {
        if fp1.get_bit(i) && fp2.get_bit(i) {
            intersection += 1;
        }
    }

    let denominator = count1 + count2;
    if denominator == 0 {
        return 0.0;
    }

    2.0 * intersection as f64 / denominator as f64
}

/// Cosine similarity
pub fn cosine_similarity(fp1: &Fingerprint, fp2: &Fingerprint) -> f64 {
    if fp1.size != fp2.size {
        return 0.0;
    }

    let mut dot_product: f64 = 0.0;
    let mut norm1: f64 = 0.0;
    let mut norm2: f64 = 0.0;

    for i in 0..fp1.size {
        let b1 = if fp1.get_bit(i) { 1.0 } else { 0.0 };
        let b2 = if fp2.get_bit(i) { 1.0 } else { 0.0 };

        dot_product += b1 * b2;
        norm1 += b1 * b1;
        norm2 += b2 * b2;
    }

    let denominator = (norm1 * norm2).sqrt();
    if denominator == 0.0 {
        return 0.0;
    }

    dot_product / denominator
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Fingerprint, FingerprintType};

    #[test]
    fn test_tanimoto_identical() {
        let mut fp = Fingerprint::new(FingerprintType::ECFP4, 1024);
        fp.set_bit(0);
        fp.set_bit(10);
        fp.set_bit(100);

        assert_eq!(tanimoto_similarity(&fp, &fp), 1.0);
    }

    #[test]
    fn test_tanimoto_disjoint() {
        let mut fp1 = Fingerprint::new(FingerprintType::ECFP4, 1024);
        fp1.set_bit(0);

        let mut fp2 = Fingerprint::new(FingerprintType::ECFP4, 1024);
        fp2.set_bit(1);

        assert_eq!(tanimoto_similarity(&fp1, &fp2), 0.0);
    }
}
