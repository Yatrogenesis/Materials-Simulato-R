//! Material types and structures

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::property::Property;

/// Represents a material in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    /// Unique identifier
    pub id: Uuid,

    /// Chemical formula (reduced, Hill notation)
    pub formula: String,

    /// Crystal structure
    pub structure: Structure,

    /// Calculated properties
    pub properties: HashMap<String, Property>,

    /// Metadata
    pub metadata: Metadata,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Crystal structure representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Structure {
    /// Lattice matrix (3x3)
    pub lattice: [[f64; 3]; 3],

    /// Atomic sites
    pub sites: Vec<Site>,

    /// Space group number (1-230)
    pub space_group: Option<u16>,

    /// Crystal system
    pub crystal_system: Option<CrystalSystem>,
}

/// Atomic site in crystal structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Site {
    /// Element symbol
    pub element: String,

    /// Fractional coordinates [x, y, z]
    pub coords: [f64; 3],

    /// Magnetic moment (optional)
    pub magmom: Option<f64>,

    /// Site occupancy
    pub occupancy: f64,
}

/// Crystal system classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CrystalSystem {
    Triclinic,
    Monoclinic,
    Orthorhombic,
    Tetragonal,
    Trigonal,
    Hexagonal,
    Cubic,
}

/// Material metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    /// Data source (MP, OQMD, AFLOW, computed, experiment)
    pub source: String,

    /// Source-specific ID (e.g., "mp-149")
    pub source_id: Option<String>,

    /// Computation method used
    pub method: Option<String>,

    /// References (DOIs, citations)
    pub references: Vec<String>,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// Additional custom metadata
    pub extra: HashMap<String, serde_json::Value>,
}

impl Material {
    /// Create a new material with minimal information
    pub fn new(formula: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            formula: formula.into(),
            structure: Structure::default(),
            properties: HashMap::new(),
            metadata: Metadata::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Get a property by name
    pub fn get_property(&self, name: &str) -> Option<&Property> {
        self.properties.get(name)
    }

    /// Add or update a property
    pub fn set_property(&mut self, name: impl Into<String>, property: Property) {
        self.properties.insert(name.into(), property);
        self.updated_at = Utc::now();
    }

    /// Get number of atoms in structure
    pub fn num_atoms(&self) -> usize {
        self.structure.sites.len()
    }

    /// Get unique elements in structure
    pub fn elements(&self) -> Vec<String> {
        let mut elements: Vec<_> = self.structure.sites
            .iter()
            .map(|site| site.element.clone())
            .collect();
        elements.sort();
        elements.dedup();
        elements
    }

    /// Calculate average atomic mass
    pub fn average_atomic_mass(&self) -> f64 {
        if self.structure.sites.is_empty() {
            return 0.0;
        }

        let total_mass: f64 = self.structure.sites
            .iter()
            .map(|site| Self::atomic_mass(&site.element))
            .sum();

        total_mass / self.structure.sites.len() as f64
    }

    /// Get atomic mass for an element (simplified lookup table)
    fn atomic_mass(element: &str) -> f64 {
        match element {
            "H" => 1.008,
            "He" => 4.003,
            "Li" => 6.941,
            "Be" => 9.012,
            "B" => 10.811,
            "C" => 12.011,
            "N" => 14.007,
            "O" => 15.999,
            "F" => 18.998,
            "Ne" => 20.180,
            "Na" => 22.990,
            "Mg" => 24.305,
            "Al" => 26.982,
            "Si" => 28.085,
            "P" => 30.974,
            "S" => 32.065,
            "Cl" => 35.453,
            "Ar" => 39.948,
            "K" => 39.098,
            "Ca" => 40.078,
            "Ti" => 47.867,
            "V" => 50.942,
            "Cr" => 51.996,
            "Mn" => 54.938,
            "Fe" => 55.845,
            "Co" => 58.933,
            "Ni" => 58.693,
            "Cu" => 63.546,
            "Zn" => 65.380,
            "Ga" => 69.723,
            "Sr" => 87.620,
            "Y" => 88.906,
            "Zr" => 91.224,
            "Nb" => 92.906,
            "Mo" => 95.960,
            "Ba" => 137.327,
            "La" => 138.905,
            "Ce" => 140.116,
            "Pb" => 207.200,
            "Bi" => 208.980,
            _ => 1.0,  // Unknown element
        }
    }
}

impl Default for Structure {
    fn default() -> Self {
        Self {
            lattice: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            sites: Vec::new(),
            space_group: None,
            crystal_system: None,
        }
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            source: "unknown".to_string(),
            source_id: None,
            method: None,
            references: Vec::new(),
            tags: Vec::new(),
            extra: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_creation() {
        let material = Material::new("Fe2O3");
        assert_eq!(material.formula, "Fe2O3");
        assert_eq!(material.num_atoms(), 0);
    }

    #[test]
    fn test_property_management() {
        let mut material = Material::new("Fe2O3");
        material.set_property(
            "formation_energy",
            Property::Scalar(-8.3),
        );

        assert!(material.get_property("formation_energy").is_some());
        assert!(material.get_property("band_gap").is_none());
    }
}
