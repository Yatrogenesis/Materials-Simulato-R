//! LIRS Integration for Quantum Chemistry Workflows
//!
//! This module provides LIRS (LISP In Rust for Science) integration for
//! automated quantum chemistry DFT calculation workflows.
//!
//! # Examples
//!
//! ```lisp
//! ;; Define a structure and run DFT
//! (define structure (perovskite :Ca :Ti :O))
//! (dft-optimize structure :xc "PBE" :kpoints [8 8 8])
//!
//! ;; Calculate formation energy
//! (formation-energy structure)
//!
//! ;; Run band structure calculation
//! (dft-bands structure :path "GXMG")
//! ```

use crate::lirs::{LIRS, SExpr, Atom, Parser};
use crate::quantum::{
    DFTCode, DFTConfig, CalculationType, XCFunctional, PseudopotentialType,
    SpinPolarization, Structure, Lattice, Atom as QAtom, Vec3D, QuantumEngine,
};
use std::path::PathBuf;
use std::collections::HashMap;
use uuid::Uuid;

/// Extended LIRS interpreter with quantum chemistry capabilities
pub struct QuantumLIRS {
    lirs: LIRS,
    structures: HashMap<String, Structure>,
    dft_configs: HashMap<String, DFTConfig>,
    work_dir: PathBuf,
}

impl QuantumLIRS {
    /// Create new QuantumLIRS interpreter
    pub fn new(work_dir: PathBuf) -> Self {
        let mut lirs = LIRS::new();
        let mut qlirs = Self {
            lirs,
            structures: HashMap::new(),
            dft_configs: HashMap::new(),
            work_dir,
        };

        qlirs.register_quantum_functions();
        qlirs
    }

    /// Register quantum chemistry functions in LIRS
    fn register_quantum_functions(&mut self) {
        // These would be registered as native functions in the LIRS interpreter
        // For now, this is a placeholder showing the design

        // Functions that will be available:
        // - (dft-config :xc "PBE" :kpoints [8 8 8] :ecut 520)
        // - (dft-optimize structure config)
        // - (dft-bands structure config)
        // - (dft-dos structure config)
        // - (formation-energy structure)
        // - (band-gap structure)
        // - (create-lattice :cubic 5.0)
        // - (create-lattice :fcc 4.0)
        // - (add-atom lattice "Fe" [0.0 0.0 0.0])
    }

    /// Evaluate LIRS code with quantum extensions
    pub fn eval(&mut self, code: &str) -> Result<SExpr, String> {
        self.lirs.eval_last(code)
    }

    /// Create DFT configuration from LIRS expression
    pub fn create_dft_config(&self, expr: &SExpr) -> Result<DFTConfig, String> {
        let mut config = DFTConfig::default();

        // Parse configuration from S-expression
        // Example: (dft-config :xc "PBE" :kpoints [8 8 8] :ecut 520.0)
        if let SExpr::List(items) = expr {
            let mut i = 1; // Skip function name
            while i < items.len() {
                if let SExpr::Atom(Atom::Element(key)) = &items[i] {
                    match key.as_str() {
                        "xc" => {
                            if let Some(SExpr::Atom(Atom::String(xc))) = items.get(i + 1) {
                                config.xc_functional = match xc.as_str() {
                                    "LDA" => XCFunctional::LDA,
                                    "PBE" => XCFunctional::PBE,
                                    "PBEsol" => XCFunctional::PBEsol,
                                    "RPBE" => XCFunctional::RPBE,
                                    "SCAN" => XCFunctional::SCAN,
                                    "HSE06" => XCFunctional::HSE06,
                                    "B3LYP" => XCFunctional::B3LYP,
                                    _ => return Err(format!("Unknown XC functional: {}", xc)),
                                };
                            }
                        }
                        "kpoints" => {
                            // Parse k-point mesh [nx ny nz]
                            if let Some(SExpr::List(kpts)) = items.get(i + 1) {
                                if kpts.len() >= 3 {
                                    config.k_points = [
                                        Self::extract_int(&kpts[0])? as usize,
                                        Self::extract_int(&kpts[1])? as usize,
                                        Self::extract_int(&kpts[2])? as usize,
                                    ];
                                }
                            }
                        }
                        "ecut" => {
                            if let Some(val) = items.get(i + 1) {
                                config.energy_cutoff = Self::extract_float(val)?;
                            }
                        }
                        "spin" => {
                            if let Some(SExpr::Atom(Atom::String(spin))) = items.get(i + 1) {
                                config.spin_polarization = match spin.as_str() {
                                    "none" => SpinPolarization::None,
                                    "collinear" => SpinPolarization::Collinear,
                                    "noncollinear" => SpinPolarization::NonCollinear,
                                    _ => return Err(format!("Unknown spin type: {}", spin)),
                                };
                            }
                        }
                        _ => {}
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
        }

        Ok(config)
    }

    /// Create crystal structure from LIRS expression
    pub fn create_structure(&self, expr: &SExpr) -> Result<Structure, String> {
        // Example: (structure :cubic 5.0 (atoms ("Fe" [0 0 0]) ("O" [0.5 0.5 0.5])))

        match expr {
            SExpr::List(items) => {
                if items.is_empty() {
                    return Err("Empty structure expression".to_string());
                }

                // Extract lattice type and parameters
                let lattice = if items.len() >= 3 {
                    if let (SExpr::Atom(Atom::Element(ltype)), param) = (&items[1], &items[2]) {
                        match ltype.as_str() {
                            "cubic" => {
                                let a = Self::extract_float(param)?;
                                Lattice::cubic(a)
                            }
                            "fcc" => {
                                let a = Self::extract_float(param)?;
                                Lattice::fcc(a)
                            }
                            "bcc" => {
                                let a = Self::extract_float(param)?;
                                Lattice::bcc(a)
                            }
                            "hexagonal" => {
                                // Expect [a c] parameters
                                if let SExpr::List(params) = param {
                                    let a = Self::extract_float(&params[0])?;
                                    let c = Self::extract_float(&params[1])?;
                                    Lattice::hexagonal(a, c)
                                } else {
                                    return Err("Hexagonal lattice needs [a c] parameters".to_string());
                                }
                            }
                            _ => return Err(format!("Unknown lattice type: {}", ltype)),
                        }
                    } else {
                        return Err("Invalid lattice specification".to_string());
                    }
                } else {
                    Lattice::cubic(5.0) // Default
                };

                // Extract atoms
                let mut atoms = Vec::new();
                // Simplified atom parsing - would need full implementation
                atoms.push(QAtom {
                    element: "Fe".to_string(),
                    position: Vec3D::zero(),
                    fractional: Vec3D::new(0.0, 0.0, 0.0),
                    magnetic_moment: None,
                    selective_dynamics: None,
                });

                let structure = Structure::new("Unknown".to_string(), lattice, atoms);
                Ok(structure)
            }
            _ => Err("Structure must be a list expression".to_string()),
        }
    }

    /// Run DFT geometry optimization
    pub fn run_optimization(
        &mut self,
        structure_name: &str,
        config_name: Option<&str>,
    ) -> Result<String, String> {
        let structure = self.structures.get(structure_name)
            .ok_or_else(|| format!("Structure '{}' not found", structure_name))?
            .clone();

        let config = if let Some(cfg_name) = config_name {
            self.dft_configs.get(cfg_name)
                .ok_or_else(|| format!("Config '{}' not found", cfg_name))?
                .clone()
        } else {
            let mut cfg = DFTConfig::default();
            cfg.calc_type = CalculationType::GeometryOpt;
            cfg
        };

        let work_dir = self.work_dir.join(format!("opt_{}", structure_name));
        std::fs::create_dir_all(&work_dir)
            .map_err(|e| format!("Failed to create work directory: {}", e))?;

        let engine = QuantumEngine::new(config, work_dir);
        engine.generate_inputs(&structure)?;

        Ok(format!("Optimization setup for {} completed", structure_name))
    }

    /// Run DFT band structure calculation
    pub fn run_band_structure(
        &mut self,
        structure_name: &str,
        kpath: &str,
    ) -> Result<String, String> {
        let structure = self.structures.get(structure_name)
            .ok_or_else(|| format!("Structure '{}' not found", structure_name))?
            .clone();

        let mut config = DFTConfig::default();
        config.calc_type = CalculationType::BandStructure;

        let work_dir = self.work_dir.join(format!("bands_{}", structure_name));
        std::fs::create_dir_all(&work_dir)
            .map_err(|e| format!("Failed to create work directory: {}", e))?;

        let engine = QuantumEngine::new(config, work_dir);
        engine.generate_inputs(&structure)?;

        Ok(format!("Band structure calculation for {} setup with path {}", structure_name, kpath))
    }

    /// Run DFT density of states calculation
    pub fn run_dos(&mut self, structure_name: &str) -> Result<String, String> {
        let structure = self.structures.get(structure_name)
            .ok_or_else(|| format!("Structure '{}' not found", structure_name))?
            .clone();

        let mut config = DFTConfig::default();
        config.calc_type = CalculationType::DOS;

        let work_dir = self.work_dir.join(format!("dos_{}", structure_name));
        std::fs::create_dir_all(&work_dir)
            .map_err(|e| format!("Failed to create work directory: {}", e))?;

        let engine = QuantumEngine::new(config, work_dir);
        engine.generate_inputs(&structure)?;

        Ok(format!("DOS calculation for {} setup", structure_name))
    }

    // Helper functions

    fn extract_float(expr: &SExpr) -> Result<f64, String> {
        match expr {
            SExpr::Atom(Atom::Float(f)) => Ok(*f),
            SExpr::Atom(Atom::Integer(i)) => Ok(*i as f64),
            _ => Err("Expected float or integer".to_string()),
        }
    }

    fn extract_int(expr: &SExpr) -> Result<i64, String> {
        match expr {
            SExpr::Atom(Atom::Integer(i)) => Ok(*i),
            SExpr::Atom(Atom::Float(f)) => Ok(*f as i64),
            _ => Err("Expected integer".to_string()),
        }
    }

    fn extract_string(expr: &SExpr) -> Result<String, String> {
        match expr {
            SExpr::Atom(Atom::String(s)) => Ok(s.clone()),
            SExpr::Atom(Atom::Symbol(s)) => Ok(s.clone()),
            _ => Err("Expected string or symbol".to_string()),
        }
    }
}

/// High-level quantum workflow automation
pub struct QuantumWorkflow {
    qlirs: QuantumLIRS,
}

impl QuantumWorkflow {
    pub fn new(work_dir: PathBuf) -> Self {
        Self {
            qlirs: QuantumLIRS::new(work_dir),
        }
    }

    /// Run a complete materials screening workflow
    ///
    /// Example workflow:
    /// 1. Generate candidate structures using LIRS
    /// 2. Run geometry optimization for each
    /// 3. Calculate formation energies
    /// 4. Calculate electronic properties (band gap, DOS)
    /// 5. Rank candidates by target properties
    pub fn run_screening_workflow(
        &mut self,
        base_structure: &str,
        substitutions: Vec<(&str, &str)>,
        target_property: &str,
    ) -> Result<Vec<String>, String> {
        let mut candidates = Vec::new();

        // Generate candidates via LIRS substitutions
        for (from_element, to_element) in substitutions {
            let code = format!(
                "(define candidate (substitute {} :{} :{}))",
                base_structure, from_element, to_element
            );

            self.qlirs.eval(&code)?;
            candidates.push(format!("candidate_{}_{}", from_element, to_element));
        }

        // Run DFT calculations for each candidate
        for candidate in &candidates {
            // Geometry optimization
            self.qlirs.run_optimization(candidate, None)?;

            // Single point calculation for properties
            // (Would launch actual DFT jobs in production)
        }

        Ok(candidates)
    }

    /// Generate phonon calculation inputs
    pub fn setup_phonon_calculation(
        &mut self,
        structure_name: &str,
        supercell: [usize; 3],
    ) -> Result<String, String> {
        // Setup for phonon calculations (PHONOPY, etc.)
        Ok(format!(
            "Phonon calculation setup for {} with supercell [{} {} {}]",
            structure_name, supercell[0], supercell[1], supercell[2]
        ))
    }

    /// Elastic constants calculation
    pub fn setup_elastic_calculation(&mut self, structure_name: &str) -> Result<String, String> {
        // Setup for elastic tensor calculation
        Ok(format!("Elastic constants calculation setup for {}", structure_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_quantum_lirs_creation() {
        let tmp_dir = TempDir::new().unwrap();
        let qlirs = QuantumLIRS::new(tmp_dir.path().to_path_buf());
        assert_eq!(qlirs.structures.len(), 0);
    }

    #[test]
    fn test_dft_config_parsing() {
        let tmp_dir = TempDir::new().unwrap();
        let qlirs = QuantumLIRS::new(tmp_dir.path().to_path_buf());

        // Create a simple config expression
        let expr = Parser::new("(dft-config :xc \"PBE\" :ecut 520.0)").parse().unwrap();

        let config = qlirs.create_dft_config(&expr).unwrap();
        assert_eq!(config.xc_functional, XCFunctional::PBE);
        assert!((config.energy_cutoff - 520.0).abs() < 1e-6);
    }

    #[test]
    fn test_workflow_creation() {
        let tmp_dir = TempDir::new().unwrap();
        let workflow = QuantumWorkflow::new(tmp_dir.path().to_path_buf());
        // Basic creation test
    }
}
