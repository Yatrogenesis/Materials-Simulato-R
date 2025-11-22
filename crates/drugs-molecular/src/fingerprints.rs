//! Molecular fingerprints for similarity calculations

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FingerprintType {
    ECFP4,    // Extended Connectivity Fingerprint (radius 2)
    ECFP6,    // Extended Connectivity Fingerprint (radius 3)
    MACCS,    // 166-bit MACCS keys
    Morgan,   // Morgan fingerprint
    Daylight, // Daylight-style fingerprint
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fingerprint {
    pub fp_type: FingerprintType,
    pub bits: Vec<bool>,
    pub size: usize,
}

impl Fingerprint {
    pub fn new(fp_type: FingerprintType, size: usize) -> Self {
        Fingerprint {
            fp_type,
            bits: vec![false; size],
            size,
        }
    }

    pub fn set_bit(&mut self, index: usize) {
        if index < self.size {
            self.bits[index] = true;
        }
    }

    pub fn get_bit(&self, index: usize) -> bool {
        self.bits.get(index).copied().unwrap_or(false)
    }

    pub fn count_on_bits(&self) -> usize {
        self.bits.iter().filter(|&&b| b).count()
    }
}
