//! DFT bridges (VASP, Quantum ESPRESSO, etc.)

pub struct DFTBridge {
    // TODO: Add DFT configuration
}

impl DFTBridge {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for DFTBridge {
    fn default() -> Self {
        Self::new()
    }
}
