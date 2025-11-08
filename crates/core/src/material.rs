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
