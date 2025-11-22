//! Molecule representation and operations
//!
//! Based on RDKit functionality but implemented in pure Rust

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Molecule {
    pub id: Uuid,
    pub smiles: String,
    pub formula: Option<String>,
    pub molecular_weight: Option<f64>,
    pub properties: HashMap<String, f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Molecule {
    pub fn new(smiles: impl Into<String>) -> Self {
        let now = Utc::now();
        Molecule {
            id: Uuid::new_v4(),
            smiles: smiles.into(),
            formula: None,
            molecular_weight: None,
            properties: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_formula(mut self, formula: impl Into<String>) -> Self {
        self.formula = Some(formula.into());
        self
    }

    pub fn with_molecular_weight(mut self, mw: f64) -> Self {
        self.molecular_weight = Some(mw);
        self
    }

    pub fn set_property(&mut self, name: impl Into<String>, value: f64) {
        self.properties.insert(name.into(), value);
        self.updated_at = Utc::now();
    }

    pub fn get_property(&self, name: &str) -> Option<f64> {
        self.properties.get(name).copied()
    }
}
