//! Quantum Chemistry DFT Integration Module
//!
//! This module provides comprehensive Density Functional Theory (DFT) integration
//! for ab-initio quantum mechanical calculations of material properties.
//!
//! # Supported DFT Codes
//! - VASP (Vienna Ab initio Simulation Package)
//! - Quantum ESPRESSO
//! - GPAW (Grid-based Projector Augmented Wave)
//! - CASTEP
//!
//! # Features
//! - Input file generation for multiple DFT codes
//! - Output parsing and property extraction
//! - Geometry optimization
//! - Band structure calculations
//! - Density of States (DOS)
//! - Formation energy calculations
//! - LIRS integration for automated workflows

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

// ============================================================================
// CORE TYPES
// ============================================================================

/// Supported DFT calculation codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DFTCode {
    VASP,
    QuantumESPRESSO,
    GPAW,
    CASTEP,
}

/// Type of DFT calculation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalculationType {
    SinglePoint,         // Single point energy calculation
    GeometryOpt,         // Geometry optimization
    BandStructure,       // Band structure calculation
    DOS,                 // Density of states
    MolecularDynamics,   // MD simulation
    PhononDispersion,    // Phonon calculations
    ElasticConstants,    // Elastic tensor
}

/// Exchange-correlation functional
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum XCFunctional {
    LDA,           // Local Density Approximation
    PBE,           // Perdew-Burke-Ernzerhof GGA
    PBEsol,        // PBE for solids
    RPBE,          // Revised PBE
    SCAN,          // Strongly Constrained and Appropriately Normed
    HSE06,         // Heyd-Scuseria-Ernzerhof hybrid
    B3LYP,         // Becke 3-parameter Lee-Yang-Parr
}

/// Pseudopotential type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PseudopotentialType {
    PAW,           // Projector Augmented Wave
    Ultrasoft,     // Ultrasoft pseudopotentials
    NormConserving, // Norm-conserving
}

/// Spin polarization settings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpinPolarization {
    None,          // Non-magnetic
    Collinear,     // Collinear spin
    NonCollinear,  // Non-collinear spin (SOC)
}

// ============================================================================
// ATOMIC STRUCTURE
// ============================================================================

/// 3D point in Cartesian coordinates
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vec3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3D {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn dot(&self, other: &Vec3D) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Vec3D) -> Vec3D {
        Vec3D::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(&self) -> Vec3D {
        let mag = self.magnitude();
        Vec3D::new(self.x / mag, self.y / mag, self.z / mag)
    }
}

/// Atom in a crystal structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Atom {
    pub element: String,
    pub position: Vec3D,      // Cartesian coordinates (Angstrom)
    pub fractional: Vec3D,    // Fractional coordinates
    pub magnetic_moment: Option<f64>, // Initial magnetic moment (μB)
    pub selective_dynamics: Option<[bool; 3]>, // Fix x,y,z coordinates
}

/// Unit cell / lattice vectors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lattice {
    pub a: Vec3D,  // First lattice vector
    pub b: Vec3D,  // Second lattice vector
    pub c: Vec3D,  // Third lattice vector
}

impl Lattice {
    /// Calculate volume of the unit cell
    pub fn volume(&self) -> f64 {
        self.a.dot(&self.b.cross(&self.c)).abs()
    }

    /// Get lattice parameters (a, b, c, alpha, beta, gamma)
    pub fn parameters(&self) -> (f64, f64, f64, f64, f64, f64) {
        let a = self.a.magnitude();
        let b = self.b.magnitude();
        let c = self.c.magnitude();

        let alpha = (self.b.dot(&self.c) / (b * c)).acos().to_degrees();
        let beta = (self.a.dot(&self.c) / (a * c)).acos().to_degrees();
        let gamma = (self.a.dot(&self.b) / (a * b)).acos().to_degrees();

        (a, b, c, alpha, beta, gamma)
    }

    /// Create cubic lattice
    pub fn cubic(a: f64) -> Self {
        Self {
            a: Vec3D::new(a, 0.0, 0.0),
            b: Vec3D::new(0.0, a, 0.0),
            c: Vec3D::new(0.0, 0.0, a),
        }
    }

    /// Create FCC lattice
    pub fn fcc(a: f64) -> Self {
        Self {
            a: Vec3D::new(0.0, a/2.0, a/2.0),
            b: Vec3D::new(a/2.0, 0.0, a/2.0),
            c: Vec3D::new(a/2.0, a/2.0, 0.0),
        }
    }

    /// Create BCC lattice
    pub fn bcc(a: f64) -> Self {
        Self {
            a: Vec3D::new(-a/2.0, a/2.0, a/2.0),
            b: Vec3D::new(a/2.0, -a/2.0, a/2.0),
            c: Vec3D::new(a/2.0, a/2.0, -a/2.0),
        }
    }

    /// Create hexagonal lattice
    pub fn hexagonal(a: f64, c: f64) -> Self {
        Self {
            a: Vec3D::new(a, 0.0, 0.0),
            b: Vec3D::new(-a/2.0, a * 3.0_f64.sqrt() / 2.0, 0.0),
            c: Vec3D::new(0.0, 0.0, c),
        }
    }
}

/// Complete crystal structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Structure {
    pub id: Uuid,
    pub formula: String,
    pub lattice: Lattice,
    pub atoms: Vec<Atom>,
    pub comment: Option<String>,
}

impl Structure {
    pub fn new(formula: String, lattice: Lattice, atoms: Vec<Atom>) -> Self {
        Self {
            id: Uuid::new_v4(),
            formula,
            lattice,
            atoms,
            comment: None,
        }
    }

    /// Get unique elements in structure
    pub fn elements(&self) -> Vec<String> {
        let mut elements: Vec<String> = self.atoms.iter()
            .map(|a| a.element.clone())
            .collect();
        elements.sort();
        elements.dedup();
        elements
    }

    /// Count atoms per element
    pub fn composition(&self) -> HashMap<String, usize> {
        let mut comp = HashMap::new();
        for atom in &self.atoms {
            *comp.entry(atom.element.clone()).or_insert(0) += 1;
        }
        comp
    }
}

// ============================================================================
// DFT CALCULATION CONFIGURATION
// ============================================================================

/// DFT calculation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DFTConfig {
    pub code: DFTCode,
    pub calc_type: CalculationType,
    pub xc_functional: XCFunctional,
    pub pseudopotential: PseudopotentialType,
    pub spin_polarization: SpinPolarization,

    // Numerical parameters
    pub energy_cutoff: f64,        // eV (plane wave cutoff)
    pub k_points: [usize; 3],      // k-point mesh
    pub k_point_shift: [f64; 3],   // k-point shift
    pub energy_convergence: f64,   // eV (SCF convergence)
    pub force_convergence: f64,    // eV/Å (geometry optimization)
    pub max_scf_iterations: usize,
    pub max_opt_iterations: usize,

    // Electronic structure
    pub smearing_method: String,   // "gaussian", "fermi-dirac", "mp"
    pub smearing_width: f64,       // eV
    pub nbands: Option<usize>,     // Number of bands (auto if None)

    // Advanced settings
    pub use_symmetry: bool,
    pub dipole_correction: bool,
    pub hubbard_u: HashMap<String, f64>, // DFT+U corrections
}

impl Default for DFTConfig {
    fn default() -> Self {
        Self {
            code: DFTCode::VASP,
            calc_type: CalculationType::SinglePoint,
            xc_functional: XCFunctional::PBE,
            pseudopotential: PseudopotentialType::PAW,
            spin_polarization: SpinPolarization::None,
            energy_cutoff: 520.0,
            k_points: [8, 8, 8],
            k_point_shift: [0.0, 0.0, 0.0],
            energy_convergence: 1e-6,
            force_convergence: 0.01,
            max_scf_iterations: 100,
            max_opt_iterations: 200,
            smearing_method: "gaussian".to_string(),
            smearing_width: 0.05,
            nbands: None,
            use_symmetry: true,
            dipole_correction: false,
            hubbard_u: HashMap::new(),
        }
    }
}

// ============================================================================
// DFT RESULTS
// ============================================================================

/// Results from a DFT calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DFTResult {
    pub id: Uuid,
    pub structure_id: Uuid,
    pub calc_type: CalculationType,
    pub converged: bool,

    // Energies
    pub total_energy: Option<f64>,        // eV
    pub energy_per_atom: Option<f64>,     // eV/atom
    pub formation_energy: Option<f64>,    // eV/atom
    pub fermi_energy: Option<f64>,        // eV

    // Electronic properties
    pub band_gap: Option<f64>,            // eV
    pub is_metal: Option<bool>,
    pub magnetic_moment: Option<f64>,     // μB

    // Forces and stresses
    pub forces: Option<Vec<Vec3D>>,       // eV/Å
    pub stress_tensor: Option<[[f64; 3]; 3]>, // GPa
    pub pressure: Option<f64>,            // GPa

    // Optimized structure
    pub final_structure: Option<Structure>,

    // DOS and band structure (stored as file paths)
    pub dos_file: Option<PathBuf>,
    pub band_structure_file: Option<PathBuf>,

    // Metadata
    pub scf_iterations: usize,
    pub wall_time: f64,                   // seconds
    pub timestamp: String,
}

impl DFTResult {
    pub fn new(structure_id: Uuid, calc_type: CalculationType) -> Self {
        Self {
            id: Uuid::new_v4(),
            structure_id,
            calc_type,
            converged: false,
            total_energy: None,
            energy_per_atom: None,
            formation_energy: None,
            fermi_energy: None,
            band_gap: None,
            is_metal: None,
            magnetic_moment: None,
            forces: None,
            stress_tensor: None,
            pressure: None,
            final_structure: None,
            dos_file: None,
            band_structure_file: None,
            scf_iterations: 0,
            wall_time: 0.0,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

// ============================================================================
// INPUT GENERATORS
// ============================================================================

/// Generate VASP input files
pub struct VASPInputGenerator;

impl VASPInputGenerator {
    /// Generate POSCAR file (structure)
    pub fn generate_poscar(structure: &Structure) -> String {
        let mut lines = vec![];

        // Comment line
        lines.push(structure.comment.clone()
            .unwrap_or_else(|| structure.formula.clone()));

        // Scaling factor
        lines.push("1.0".to_string());

        // Lattice vectors
        lines.push(format!("  {:.10}  {:.10}  {:.10}",
            structure.lattice.a.x, structure.lattice.a.y, structure.lattice.a.z));
        lines.push(format!("  {:.10}  {:.10}  {:.10}",
            structure.lattice.b.x, structure.lattice.b.y, structure.lattice.b.z));
        lines.push(format!("  {:.10}  {:.10}  {:.10}",
            structure.lattice.c.x, structure.lattice.c.y, structure.lattice.c.z));

        // Element names and counts
        let elements = structure.elements();
        let composition = structure.composition();
        let element_line = elements.join(" ");
        let count_line = elements.iter()
            .map(|e| composition.get(e).unwrap().to_string())
            .collect::<Vec<_>>()
            .join(" ");

        lines.push(element_line);
        lines.push(count_line);

        // Selective dynamics (if any atom has it)
        let has_selective = structure.atoms.iter().any(|a| a.selective_dynamics.is_some());
        if has_selective {
            lines.push("Selective dynamics".to_string());
        }

        // Coordinate mode
        lines.push("Direct".to_string());

        // Atomic positions (sorted by element)
        for element in &elements {
            for atom in &structure.atoms {
                if atom.element == *element {
                    let mut pos_line = format!("  {:.10}  {:.10}  {:.10}",
                        atom.fractional.x, atom.fractional.y, atom.fractional.z);

                    if let Some(sd) = atom.selective_dynamics {
                        let flags = sd.iter()
                            .map(|&b| if b { "T" } else { "F" })
                            .collect::<Vec<_>>()
                            .join(" ");
                        pos_line.push_str(&format!("  {}", flags));
                    }

                    lines.push(pos_line);
                }
            }
        }

        lines.join("\n") + "\n"
    }

    /// Generate INCAR file (calculation parameters)
    pub fn generate_incar(config: &DFTConfig) -> String {
        let mut lines = vec![];

        lines.push("# VASP Input File Generated by Materials-Simulato-R".to_string());
        lines.push("".to_string());

        // System
        lines.push("SYSTEM = DFT Calculation".to_string());
        lines.push("".to_string());

        // Electronic minimization
        lines.push("# Electronic Minimization".to_string());
        lines.push(format!("ENCUT = {:.1}", config.energy_cutoff));
        lines.push(format!("EDIFF = {:.2e}", config.energy_convergence));
        lines.push(format!("NELM = {}", config.max_scf_iterations));
        lines.push("ALGO = Normal".to_string());
        lines.push("".to_string());

        // XC functional
        let gga = match config.xc_functional {
            XCFunctional::PBE => "PE",
            XCFunctional::PBEsol => "PS",
            XCFunctional::RPBE => "RP",
            XCFunctional::LDA => "",
            _ => "PE",
        };
        if !gga.is_empty() {
            lines.push(format!("GGA = {}", gga));
        }

        // Smearing
        lines.push(format!("ISMEAR = {}", match config.smearing_method.as_str() {
            "gaussian" => "0",
            "fermi-dirac" => "-1",
            "mp" | "methfessel-paxton" => "1",
            _ => "0",
        }));
        lines.push(format!("SIGMA = {:.3}", config.smearing_width));
        lines.push("".to_string());

        // Spin polarization
        match config.spin_polarization {
            SpinPolarization::Collinear => {
                lines.push("ISPIN = 2".to_string());
            }
            SpinPolarization::NonCollinear => {
                lines.push("ISPIN = 2".to_string());
                lines.push("LNONCOLLINEAR = .TRUE.".to_string());
                lines.push("LSORBIT = .TRUE.".to_string());
            }
            SpinPolarization::None => {
                lines.push("ISPIN = 1".to_string());
            }
        }
        lines.push("".to_string());

        // Calculation type specific settings
        match config.calc_type {
            CalculationType::GeometryOpt => {
                lines.push("# Geometry Optimization".to_string());
                lines.push("IBRION = 2".to_string());
                lines.push("ISIF = 3".to_string());
                lines.push(format!("EDIFFG = -{:.3}", config.force_convergence));
                lines.push(format!("NSW = {}", config.max_opt_iterations));
            }
            CalculationType::BandStructure => {
                lines.push("# Band Structure".to_string());
                lines.push("ICHARG = 11".to_string());
                lines.push("LORBIT = 11".to_string());
            }
            CalculationType::DOS => {
                lines.push("# Density of States".to_string());
                lines.push("LORBIT = 11".to_string());
                lines.push("NEDOS = 2001".to_string());
            }
            _ => {}
        }
        lines.push("".to_string());

        // Symmetry
        if config.use_symmetry {
            lines.push("ISYM = 2".to_string());
        } else {
            lines.push("ISYM = 0".to_string());
        }

        // Dipole correction
        if config.dipole_correction {
            lines.push("LDIPOL = .TRUE.".to_string());
            lines.push("IDIPOL = 3".to_string());
        }

        // DFT+U
        if !config.hubbard_u.is_empty() {
            lines.push("".to_string());
            lines.push("# DFT+U".to_string());
            lines.push("LDAU = .TRUE.".to_string());
            lines.push("LDAUTYPE = 2".to_string());
            // Note: LDAUU and LDAUL need element ordering - simplified here
        }

        // Output
        lines.push("".to_string());
        lines.push("# Output".to_string());
        lines.push("LWAVE = .FALSE.".to_string());
        lines.push("LCHARG = .FALSE.".to_string());

        lines.join("\n") + "\n"
    }

    /// Generate KPOINTS file
    pub fn generate_kpoints(config: &DFTConfig) -> String {
        let mut lines = vec![];

        lines.push("Automatic mesh".to_string());
        lines.push("0".to_string());
        lines.push("Gamma".to_string());
        lines.push(format!("  {}  {}  {}",
            config.k_points[0], config.k_points[1], config.k_points[2]));
        lines.push(format!("  {:.3}  {:.3}  {:.3}",
            config.k_point_shift[0], config.k_point_shift[1], config.k_point_shift[2]));

        lines.join("\n") + "\n"
    }
}

// ============================================================================
// OUTPUT PARSERS
// ============================================================================

/// Parse VASP OUTCAR file
pub struct VASPOutputParser;

impl VASPOutputParser {
    /// Parse final energy from OUTCAR
    pub fn parse_energy(outcar_content: &str) -> Option<f64> {
        for line in outcar_content.lines().rev() {
            if line.contains("energy  without entropy") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(energy_str) = parts.get(parts.len() - 1) {
                    return energy_str.parse::<f64>().ok();
                }
            }
        }
        None
    }

    /// Parse forces from OUTCAR
    pub fn parse_forces(outcar_content: &str) -> Option<Vec<Vec3D>> {
        let mut forces = Vec::new();
        let mut in_forces_section = false;

        for line in outcar_content.lines() {
            if line.contains("POSITION") && line.contains("TOTAL-FORCE") {
                in_forces_section = true;
                continue;
            }

            if in_forces_section {
                if line.trim().is_empty() || line.contains("----") {
                    if !forces.is_empty() {
                        return Some(forces);
                    }
                    forces.clear();
                    in_forces_section = false;
                    continue;
                }

                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 6 {
                    if let (Ok(fx), Ok(fy), Ok(fz)) = (
                        parts[3].parse::<f64>(),
                        parts[4].parse::<f64>(),
                        parts[5].parse::<f64>(),
                    ) {
                        forces.push(Vec3D::new(fx, fy, fz));
                    }
                }
            }
        }

        None
    }

    /// Parse band gap from OUTCAR
    pub fn parse_band_gap(outcar_content: &str) -> Option<f64> {
        // Simple implementation - would need more sophisticated parsing
        // for real band gap detection (VBM and CBM)
        None
    }

    /// Check if calculation converged
    pub fn is_converged(outcar_content: &str) -> bool {
        outcar_content.contains("reached required accuracy")
    }
}

// ============================================================================
// QUANTUM ENGINE
// ============================================================================

/// Main quantum chemistry calculation engine
#[derive(Debug)]
pub struct QuantumEngine {
    config: DFTConfig,
    work_dir: PathBuf,
}

impl QuantumEngine {
    pub fn new(config: DFTConfig, work_dir: PathBuf) -> Self {
        Self { config, work_dir }
    }

    /// Generate input files for a calculation
    pub fn generate_inputs(&self, structure: &Structure) -> Result<(), String> {
        match self.config.code {
            DFTCode::VASP => {
                let poscar = VASPInputGenerator::generate_poscar(structure);
                let incar = VASPInputGenerator::generate_incar(&self.config);
                let kpoints = VASPInputGenerator::generate_kpoints(&self.config);

                std::fs::write(self.work_dir.join("POSCAR"), poscar)
                    .map_err(|e| format!("Failed to write POSCAR: {}", e))?;
                std::fs::write(self.work_dir.join("INCAR"), incar)
                    .map_err(|e| format!("Failed to write INCAR: {}", e))?;
                std::fs::write(self.work_dir.join("KPOINTS"), kpoints)
                    .map_err(|e| format!("Failed to write KPOINTS: {}", e))?;

                Ok(())
            }
            _ => Err("Only VASP is currently implemented".to_string()),
        }
    }

    /// Parse calculation results
    pub fn parse_results(&self, structure_id: Uuid) -> Result<DFTResult, String> {
        match self.config.code {
            DFTCode::VASP => {
                let outcar_path = self.work_dir.join("OUTCAR");
                let content = std::fs::read_to_string(&outcar_path)
                    .map_err(|e| format!("Failed to read OUTCAR: {}", e))?;

                let mut result = DFTResult::new(structure_id, self.config.calc_type);

                result.converged = VASPOutputParser::is_converged(&content);
                result.total_energy = VASPOutputParser::parse_energy(&content);
                result.forces = VASPOutputParser::parse_forces(&content);
                result.band_gap = VASPOutputParser::parse_band_gap(&content);

                Ok(result)
            }
            _ => Err("Only VASP parsing is currently implemented".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec3d_operations() {
        let v1 = Vec3D::new(1.0, 0.0, 0.0);
        let v2 = Vec3D::new(0.0, 1.0, 0.0);

        let cross = v1.cross(&v2);
        assert!((cross.z - 1.0).abs() < 1e-10);

        let dot = v1.dot(&v2);
        assert!((dot - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_lattice_volume() {
        let cubic = Lattice::cubic(5.0);
        assert!((cubic.volume() - 125.0).abs() < 1e-10);
    }

    #[test]
    fn test_vasp_poscar_generation() {
        let lattice = Lattice::cubic(4.0);
        let atoms = vec![
            Atom {
                element: "Fe".to_string(),
                position: Vec3D::zero(),
                fractional: Vec3D::new(0.0, 0.0, 0.0),
                magnetic_moment: Some(4.0),
                selective_dynamics: None,
            },
        ];

        let structure = Structure::new("Fe".to_string(), lattice, atoms);
        let poscar = VASPInputGenerator::generate_poscar(&structure);

        assert!(poscar.contains("Fe"));
        assert!(poscar.contains("Direct"));
    }

    #[test]
    fn test_vasp_incar_generation() {
        let config = DFTConfig::default();
        let incar = VASPInputGenerator::generate_incar(&config);

        assert!(incar.contains("ENCUT"));
        assert!(incar.contains("EDIFF"));
        assert!(incar.contains("GGA"));
    }

    #[test]
    fn test_structure_composition() {
        let lattice = Lattice::cubic(5.0);
        let atoms = vec![
            Atom {
                element: "Fe".to_string(),
                position: Vec3D::zero(),
                fractional: Vec3D::new(0.0, 0.0, 0.0),
                magnetic_moment: None,
                selective_dynamics: None,
            },
            Atom {
                element: "O".to_string(),
                position: Vec3D::zero(),
                fractional: Vec3D::new(0.5, 0.5, 0.5),
                magnetic_moment: None,
                selective_dynamics: None,
            },
            Atom {
                element: "O".to_string(),
                position: Vec3D::zero(),
                fractional: Vec3D::new(0.25, 0.25, 0.25),
                magnetic_moment: None,
                selective_dynamics: None,
            },
        ];

        let structure = Structure::new("FeO2".to_string(), lattice, atoms);
        let comp = structure.composition();

        assert_eq!(comp.get("Fe"), Some(&1));
        assert_eq!(comp.get("O"), Some(&2));
    }
}
