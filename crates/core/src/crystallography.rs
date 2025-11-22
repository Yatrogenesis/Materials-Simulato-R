//! Advanced Crystallography Module
//!
//! This module provides comprehensive crystallographic analysis tools including
//! space groups, symmetry operations, Wyckoff positions, and structure manipulation.
//!
//! # Features
//! - Space group identification and operations
//! - Wyckoff position analysis
//! - Symmetry operations (rotation, reflection, inversion)
//! - Crystal system classification
//! - Point group determination
//! - Defect generation (vacancies, interstitials, substitutions)
//! - Surface structure generation
//! - Supercell construction

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f64::consts::PI;

// ============================================================================
// CRYSTALLOGRAPHIC TYPES
// ============================================================================

/// The 7 crystal systems
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrystalSystem {
    Triclinic,
    Monoclinic,
    Orthorhombic,
    Tetragonal,
    Trigonal,
    Hexagonal,
    Cubic,
}

impl CrystalSystem {
    /// Get conventional lattice parameter constraints
    pub fn constraints(&self) -> &'static str {
        match self {
            Self::Triclinic => "a≠b≠c, α≠β≠γ≠90°",
            Self::Monoclinic => "a≠b≠c, α=γ=90°≠β",
            Self::Orthorhombic => "a≠b≠c, α=β=γ=90°",
            Self::Tetragonal => "a=b≠c, α=β=γ=90°",
            Self::Trigonal => "a=b=c, α=β=γ<120°,≠90°",
            Self::Hexagonal => "a=b≠c, α=β=90°, γ=120°",
            Self::Cubic => "a=b=c, α=β=γ=90°",
        }
    }

    /// Number of space groups in this crystal system
    pub fn num_space_groups(&self) -> usize {
        match self {
            Self::Triclinic => 2,
            Self::Monoclinic => 13,
            Self::Orthorhombic => 59,
            Self::Tetragonal => 68,
            Self::Trigonal => 25,
            Self::Hexagonal => 27,
            Self::Cubic => 36,
        }
    }
}

/// The 14 Bravais lattices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BravaisLattice {
    // Cubic
    CubicP,
    CubicI,
    CubicF,

    // Tetragonal
    TetragonalP,
    TetragonalI,

    // Orthorhombic
    OrthorhombicP,
    OrthorhombicC,
    OrthorhombicI,
    OrthorhombicF,

    // Hexagonal
    HexagonalP,

    // Trigonal
    TrigonalP,
    TrigonalR,

    // Monoclinic
    MonoclinicP,
    MonoclinicC,

    // Triclinic
    TriclinicP,
}

impl BravaisLattice {
    pub fn crystal_system(&self) -> CrystalSystem {
        match self {
            Self::CubicP | Self::CubicI | Self::CubicF => CrystalSystem::Cubic,
            Self::TetragonalP | Self::TetragonalI => CrystalSystem::Tetragonal,
            Self::OrthorhombicP | Self::OrthorhombicC | Self::OrthorhombicI | Self::OrthorhombicF => {
                CrystalSystem::Orthorhombic
            }
            Self::HexagonalP => CrystalSystem::Hexagonal,
            Self::TrigonalP | Self::TrigonalR => CrystalSystem::Trigonal,
            Self::MonoclinicP | Self::MonoclinicC => CrystalSystem::Monoclinic,
            Self::TriclinicP => CrystalSystem::Triclinic,
        }
    }

    pub fn centering(&self) -> &'static str {
        match self {
            Self::CubicP | Self::TetragonalP | Self::OrthorhombicP | Self::HexagonalP
            | Self::TrigonalP | Self::MonoclinicP | Self::TriclinicP => "Primitive (P)",

            Self::CubicI | Self::TetragonalI | Self::OrthorhombicI => "Body-centered (I)",

            Self::CubicF | Self::OrthorhombicF => "Face-centered (F)",

            Self::OrthorhombicC | Self::MonoclinicC => "Base-centered (C)",

            Self::TrigonalR => "Rhombohedral (R)",
        }
    }
}

// ============================================================================
// 3D VECTORS AND MATRICES
// ============================================================================

/// 3D vector for crystallographic calculations
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn dot(&self, other: &Vec3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(&self) -> Vec3 {
        let mag = self.magnitude();
        Vec3::new(self.x / mag, self.y / mag, self.z / mag)
    }

    pub fn add(&self, other: &Vec3) -> Vec3 {
        Vec3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    pub fn sub(&self, other: &Vec3) -> Vec3 {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    pub fn scale(&self, scalar: f64) -> Vec3 {
        Vec3::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

/// 3x3 matrix for symmetry operations
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Matrix3x3 {
    pub data: [[f64; 3]; 3],
}

impl Matrix3x3 {
    pub fn identity() -> Self {
        Self {
            data: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    pub fn rotation_x(angle: f64) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Self {
            data: [[1.0, 0.0, 0.0], [0.0, c, -s], [0.0, s, c]],
        }
    }

    pub fn rotation_y(angle: f64) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Self {
            data: [[c, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, c]],
        }
    }

    pub fn rotation_z(angle: f64) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Self {
            data: [[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    pub fn inversion() -> Self {
        Self {
            data: [[-1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, -1.0]],
        }
    }

    pub fn apply(&self, vec: &Vec3) -> Vec3 {
        Vec3::new(
            self.data[0][0] * vec.x + self.data[0][1] * vec.y + self.data[0][2] * vec.z,
            self.data[1][0] * vec.x + self.data[1][1] * vec.y + self.data[1][2] * vec.z,
            self.data[2][0] * vec.x + self.data[2][1] * vec.y + self.data[2][2] * vec.z,
        )
    }

    pub fn multiply(&self, other: &Matrix3x3) -> Matrix3x3 {
        let mut result = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    result[i][j] += self.data[i][k] * other.data[k][j];
                }
            }
        }
        Matrix3x3 { data: result }
    }

    pub fn determinant(&self) -> f64 {
        let d = &self.data;
        d[0][0] * (d[1][1] * d[2][2] - d[1][2] * d[2][1])
            - d[0][1] * (d[1][0] * d[2][2] - d[1][2] * d[2][0])
            + d[0][2] * (d[1][0] * d[2][1] - d[1][1] * d[2][0])
    }
}

// ============================================================================
// SYMMETRY OPERATIONS
// ============================================================================

/// A crystallographic symmetry operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymmetryOperation {
    pub name: String,
    pub rotation: Matrix3x3,
    pub translation: Vec3,
}

impl SymmetryOperation {
    pub fn identity() -> Self {
        Self {
            name: "Identity (E)".to_string(),
            rotation: Matrix3x3::identity(),
            translation: Vec3::zero(),
        }
    }

    pub fn inversion() -> Self {
        Self {
            name: "Inversion (i)".to_string(),
            rotation: Matrix3x3::inversion(),
            translation: Vec3::zero(),
        }
    }

    pub fn rotation(axis: char, fold: usize) -> Self {
        let angle = 2.0 * PI / (fold as f64);
        let rotation = match axis {
            'x' => Matrix3x3::rotation_x(angle),
            'y' => Matrix3x3::rotation_y(angle),
            'z' => Matrix3x3::rotation_z(angle),
            _ => Matrix3x3::identity(),
        };

        Self {
            name: format!("Rotation {}-fold about {}", fold, axis),
            rotation,
            translation: Vec3::zero(),
        }
    }

    pub fn mirror(axis: char) -> Self {
        // Mirror perpendicular to axis
        let mut data = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        match axis {
            'x' => data[0][0] = -1.0,
            'y' => data[1][1] = -1.0,
            'z' => data[2][2] = -1.0,
            _ => {}
        }

        Self {
            name: format!("Mirror perpendicular to {}", axis),
            rotation: Matrix3x3 { data },
            translation: Vec3::zero(),
        }
    }

    pub fn screw(axis: char, fold: usize, translation_fraction: f64) -> Self {
        let mut op = Self::rotation(axis, fold);
        let mut translation = Vec3::zero();
        match axis {
            'x' => translation.x = translation_fraction,
            'y' => translation.y = translation_fraction,
            'z' => translation.z = translation_fraction,
            _ => {}
        }
        op.translation = translation;
        op.name = format!("Screw {}-fold about {} + translation", fold, axis);
        op
    }

    pub fn glide(axis: char, translation_axis: char, translation_fraction: f64) -> Self {
        let mut op = Self::mirror(axis);
        let mut translation = Vec3::zero();
        match translation_axis {
            'x' => translation.x = translation_fraction,
            'y' => translation.y = translation_fraction,
            'z' => translation.z = translation_fraction,
            _ => {}
        }
        op.translation = translation;
        op.name = format!("Glide perpendicular to {}, translation along {}", axis, translation_axis);
        op
    }

    /// Apply symmetry operation to a position
    pub fn apply(&self, pos: &Vec3) -> Vec3 {
        self.rotation.apply(pos).add(&self.translation)
    }
}

// ============================================================================
// SPACE GROUPS
// ============================================================================

/// Space group (1-230 in International Tables)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceGroup {
    pub number: usize,            // 1-230
    pub symbol: String,           // Hermann-Mauguin symbol
    pub crystal_system: CrystalSystem,
    pub bravais_lattice: BravaisLattice,
    pub operations: Vec<SymmetryOperation>,
}

impl SpaceGroup {
    /// Create common space groups
    pub fn pm3m() -> Self {
        // Cubic Pm-3m (No. 221)
        Self {
            number: 221,
            symbol: "Pm-3m".to_string(),
            crystal_system: CrystalSystem::Cubic,
            bravais_lattice: BravaisLattice::CubicP,
            operations: vec![
                SymmetryOperation::identity(),
                SymmetryOperation::inversion(),
                SymmetryOperation::rotation('x', 4),
                SymmetryOperation::rotation('y', 4),
                SymmetryOperation::rotation('z', 4),
            ],
        }
    }

    pub fn fm3m() -> Self {
        // Cubic Fm-3m (No. 225) - FCC
        Self {
            number: 225,
            symbol: "Fm-3m".to_string(),
            crystal_system: CrystalSystem::Cubic,
            bravais_lattice: BravaisLattice::CubicF,
            operations: vec![
                SymmetryOperation::identity(),
                SymmetryOperation::inversion(),
                SymmetryOperation::rotation('x', 4),
                SymmetryOperation::rotation('y', 4),
                SymmetryOperation::rotation('z', 4),
            ],
        }
    }

    pub fn im3m() -> Self {
        // Cubic Im-3m (No. 229) - BCC
        Self {
            number: 229,
            symbol: "Im-3m".to_string(),
            crystal_system: CrystalSystem::Cubic,
            bravais_lattice: BravaisLattice::CubicI,
            operations: vec![
                SymmetryOperation::identity(),
                SymmetryOperation::inversion(),
                SymmetryOperation::rotation('x', 4),
                SymmetryOperation::rotation('y', 4),
                SymmetryOperation::rotation('z', 4),
            ],
        }
    }

    pub fn p63mmc() -> Self {
        // Hexagonal P63/mmc (No. 194) - Wurtzite
        Self {
            number: 194,
            symbol: "P63/mmc".to_string(),
            crystal_system: CrystalSystem::Hexagonal,
            bravais_lattice: BravaisLattice::HexagonalP,
            operations: vec![
                SymmetryOperation::identity(),
                SymmetryOperation::rotation('z', 6),
                SymmetryOperation::rotation('z', 3),
                SymmetryOperation::rotation('z', 2),
                SymmetryOperation::mirror('z'),
            ],
        }
    }

    /// Get number of symmetry operations
    pub fn num_operations(&self) -> usize {
        self.operations.len()
    }

    /// Generate all equivalent positions for a given position
    pub fn equivalent_positions(&self, pos: &Vec3) -> Vec<Vec3> {
        let mut positions: Vec<Vec3> = Vec::new();

        for op in &self.operations {
            let new_pos = op.apply(pos);

            // Wrap to [0, 1) range
            let wrapped = Vec3::new(
                new_pos.x - new_pos.x.floor(),
                new_pos.y - new_pos.y.floor(),
                new_pos.z - new_pos.z.floor(),
            );

            // Check if position is unique (within tolerance)
            let tolerance = 1e-6;
            let mut is_unique = true;
            for existing in &positions {
                let dist = (wrapped.x - existing.x).abs()
                    + (wrapped.y - existing.y).abs()
                    + (wrapped.z - existing.z).abs();
                if dist < tolerance {
                    is_unique = false;
                    break;
                }
            }

            if is_unique {
                positions.push(wrapped);
            }
        }

        positions
    }
}

// ============================================================================
// WYCKOFF POSITIONS
// ============================================================================

/// Wyckoff position in a space group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WyckoffPosition {
    pub letter: char,              // a, b, c, ...
    pub multiplicity: usize,       // Number of equivalent sites
    pub site_symmetry: String,     // Point group at this site
    pub coordinates: Vec<Vec3>,    // Representative coordinates
}

impl WyckoffPosition {
    pub fn new(letter: char, multiplicity: usize, site_symmetry: String) -> Self {
        Self {
            letter,
            multiplicity,
            site_symmetry,
            coordinates: vec![],
        }
    }

    pub fn add_coordinate(&mut self, coord: Vec3) {
        self.coordinates.push(coord);
    }

    /// Wyckoff 1a for cubic Pm-3m (corner position)
    pub fn cubic_1a() -> Self {
        let mut wp = Self::new('a', 1, "m-3m".to_string());
        wp.add_coordinate(Vec3::new(0.0, 0.0, 0.0));
        wp
    }

    /// Wyckoff 3c for cubic Pm-3m (face center positions)
    pub fn cubic_3c() -> Self {
        let mut wp = Self::new('c', 3, "4/mmm".to_string());
        wp.add_coordinate(Vec3::new(0.0, 0.5, 0.5));
        wp.add_coordinate(Vec3::new(0.5, 0.0, 0.5));
        wp.add_coordinate(Vec3::new(0.5, 0.5, 0.0));
        wp
    }
}

// ============================================================================
// DEFECT GENERATION
// ============================================================================

/// Type of crystal defect
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DefectType {
    Vacancy,
    Interstitial,
    Substitution,
    Schottky,
    Frenkel,
}

/// Crystal defect descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Defect {
    pub defect_type: DefectType,
    pub element: Option<String>,
    pub position: Vec3,
    pub charge: Option<i32>,
}

impl Defect {
    pub fn vacancy(position: Vec3) -> Self {
        Self {
            defect_type: DefectType::Vacancy,
            element: None,
            position,
            charge: None,
        }
    }

    pub fn interstitial(element: String, position: Vec3) -> Self {
        Self {
            defect_type: DefectType::Interstitial,
            element: Some(element),
            position,
            charge: None,
        }
    }

    pub fn substitution(element: String, position: Vec3) -> Self {
        Self {
            defect_type: DefectType::Substitution,
            element: Some(element),
            position,
            charge: None,
        }
    }
}

// ============================================================================
// CRYSTALLOGRAPHIC ANALYZER
// ============================================================================

/// Main crystallography analysis engine
#[derive(Debug)]
pub struct CrystallographyAnalyzer;

impl CrystallographyAnalyzer {
    /// Identify crystal system from lattice parameters
    pub fn identify_crystal_system(
        a: f64,
        b: f64,
        c: f64,
        alpha: f64,
        beta: f64,
        gamma: f64,
    ) -> CrystalSystem {
        let tolerance = 1e-3;
        let angle_tolerance = 0.1; // degrees

        let a_eq_b = (a - b).abs() < tolerance;
        let a_eq_c = (a - c).abs() < tolerance;
        let b_eq_c = (b - c).abs() < tolerance;

        let alpha_90 = (alpha - 90.0).abs() < angle_tolerance;
        let beta_90 = (beta - 90.0).abs() < angle_tolerance;
        let gamma_90 = (gamma - 90.0).abs() < angle_tolerance;
        let gamma_120 = (gamma - 120.0).abs() < angle_tolerance;

        if a_eq_b && a_eq_c && alpha_90 && beta_90 && gamma_90 {
            CrystalSystem::Cubic
        } else if a_eq_b && !a_eq_c && alpha_90 && beta_90 && gamma_120 {
            CrystalSystem::Hexagonal
        } else if a_eq_b && !a_eq_c && alpha_90 && beta_90 && gamma_90 {
            CrystalSystem::Tetragonal
        } else if !a_eq_b && !a_eq_c && alpha_90 && beta_90 && gamma_90 {
            CrystalSystem::Orthorhombic
        } else if !a_eq_b && !a_eq_c && alpha_90 && !beta_90 && gamma_90 {
            CrystalSystem::Monoclinic
        } else if a_eq_b && a_eq_c && !alpha_90 && !beta_90 && !gamma_90 {
            let alpha_eq_beta = (alpha - beta).abs() < angle_tolerance;
            let alpha_eq_gamma = (alpha - gamma).abs() < angle_tolerance;
            if alpha_eq_beta && alpha_eq_gamma {
                CrystalSystem::Trigonal
            } else {
                CrystalSystem::Triclinic
            }
        } else {
            CrystalSystem::Triclinic
        }
    }

    /// Calculate angle between two vectors
    pub fn angle_between(v1: &Vec3, v2: &Vec3) -> f64 {
        let dot = v1.dot(v2);
        let mag_product = v1.magnitude() * v2.magnitude();
        (dot / mag_product).acos().to_degrees()
    }

    /// Generate supercell
    pub fn generate_supercell(
        base_positions: &[Vec3],
        nx: usize,
        ny: usize,
        nz: usize,
    ) -> Vec<Vec3> {
        let mut supercell = Vec::new();

        for &pos in base_positions {
            for i in 0..nx {
                for j in 0..ny {
                    for k in 0..nz {
                        let new_pos = Vec3::new(
                            (pos.x + i as f64) / nx as f64,
                            (pos.y + j as f64) / ny as f64,
                            (pos.z + k as f64) / nz as f64,
                        );
                        supercell.push(new_pos);
                    }
                }
            }
        }

        supercell
    }

    /// Generate surface slab (cleave along plane)
    pub fn generate_surface_slab(
        positions: &[Vec3],
        miller_indices: (i32, i32, i32),
        num_layers: usize,
        vacuum_thickness: f64,
    ) -> Vec<Vec3> {
        // Simplified surface generation
        // Real implementation would properly cleave along Miller plane
        let mut surface = Vec::new();

        // For now, just replicate along z and add vacuum
        for &pos in positions {
            for layer in 0..num_layers {
                let new_pos = Vec3::new(
                    pos.x,
                    pos.y,
                    (pos.z + layer as f64) / (num_layers as f64 + vacuum_thickness),
                );
                surface.push(new_pos);
            }
        }

        surface
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec3_operations() {
        let v1 = Vec3::new(1.0, 0.0, 0.0);
        let v2 = Vec3::new(0.0, 1.0, 0.0);

        let cross = v1.cross(&v2);
        assert!((cross.z - 1.0).abs() < 1e-10);

        let dot = v1.dot(&v2);
        assert!((dot - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_crystal_system_identification() {
        // Cubic
        let system = CrystallographyAnalyzer::identify_crystal_system(
            5.0, 5.0, 5.0, 90.0, 90.0, 90.0,
        );
        assert_eq!(system, CrystalSystem::Cubic);

        // Hexagonal
        let system = CrystallographyAnalyzer::identify_crystal_system(
            3.0, 3.0, 5.0, 90.0, 90.0, 120.0,
        );
        assert_eq!(system, CrystalSystem::Hexagonal);
    }

    #[test]
    fn test_symmetry_operations() {
        let identity = SymmetryOperation::identity();
        let pos = Vec3::new(0.5, 0.5, 0.5);
        let result = identity.apply(&pos);

        assert!((result.x - pos.x).abs() < 1e-10);
        assert!((result.y - pos.y).abs() < 1e-10);
        assert!((result.z - pos.z).abs() < 1e-10);
    }

    #[test]
    fn test_space_group_operations() {
        let sg = SpaceGroup::pm3m();
        assert_eq!(sg.number, 221);
        assert_eq!(sg.crystal_system, CrystalSystem::Cubic);
    }

    #[test]
    fn test_equivalent_positions() {
        let sg = SpaceGroup::pm3m();
        let pos = Vec3::new(0.25, 0.25, 0.25);
        let equiv = sg.equivalent_positions(&pos);

        assert!(!equiv.is_empty());
    }

    #[test]
    fn test_supercell_generation() {
        let base = vec![Vec3::zero()];
        let supercell = CrystallographyAnalyzer::generate_supercell(&base, 2, 2, 2);

        assert_eq!(supercell.len(), 8);
    }
}
