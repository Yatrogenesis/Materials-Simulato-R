//! Biological target representation

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetType {
    Protein,
    Enzyme,
    Receptor,
    Antibody,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub id: Uuid,
    pub name: String,
    pub target_type: TargetType,
    pub sequence: Option<String>,
    pub pdb_id: Option<String>,
    pub uniprot_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Target {
    pub fn new(name: impl Into<String>, target_type: TargetType) -> Self {
        let now = Utc::now();
        Target {
            id: Uuid::new_v4(),
            name: name.into(),
            target_type,
            sequence: None,
            pdb_id: None,
            uniprot_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_sequence(mut self, sequence: impl Into<String>) -> Self {
        self.sequence = Some(sequence.into());
        self
    }

    pub fn with_pdb_id(mut self, pdb_id: impl Into<String>) -> Self {
        self.pdb_id = Some(pdb_id.into());
        self
    }

    pub fn with_uniprot_id(mut self, uniprot_id: impl Into<String>) -> Self {
        self.uniprot_id = Some(uniprot_id.into());
        self
    }
}
