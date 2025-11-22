//! Drug compound representation

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::Molecule;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Compound {
    pub id: Uuid,
    pub name: String,
    pub molecule: Molecule,
    pub drug_like_score: Option<f64>,
    pub toxicity_score: Option<f64>,
    pub solubility: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Compound {
    pub fn new(name: impl Into<String>, molecule: Molecule) -> Self {
        let now = Utc::now();
        Compound {
            id: Uuid::new_v4(),
            name: name.into(),
            molecule,
            drug_like_score: None,
            toxicity_score: None,
            solubility: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_drug_like_score(mut self, score: f64) -> Self {
        self.drug_like_score = Some(score);
        self
    }

    pub fn with_toxicity_score(mut self, score: f64) -> Self {
        self.toxicity_score = Some(score);
        self
    }

    pub fn with_solubility(mut self, solubility: f64) -> Self {
        self.solubility = Some(solubility);
        self
    }
}
