//! Property types for materials

use serde::{Deserialize, Serialize};

/// Represents a material property
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Property {
    /// Scalar value (e.g., formation energy, band gap)
    Scalar(f64),

    /// Vector value (e.g., lattice parameters)
    Vector(Vec<f64>),

    /// Matrix value (e.g., elastic constants)
    Matrix(Vec<Vec<f64>>),

    /// String value (e.g., crystal system)
    String(String),

    /// Boolean value
    Boolean(bool),

    /// JSON object for complex properties
    Object(serde_json::Value),
}

impl Property {
    /// Get scalar value if property is scalar
    pub fn as_scalar(&self) -> Option<f64> {
        match self {
            Property::Scalar(v) => Some(*v),
            _ => None,
        }
    }

    /// Get vector value if property is vector
    pub fn as_vector(&self) -> Option<&Vec<f64>> {
        match self {
            Property::Vector(v) => Some(v),
            _ => None,
        }
    }

    /// Get matrix value if property is matrix
    pub fn as_matrix(&self) -> Option<&Vec<Vec<f64>>> {
        match self {
            Property::Matrix(m) => Some(m),
            _ => None,
        }
    }

    /// Get string value if property is string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Property::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get boolean value if property is boolean
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Property::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scalar_property() {
        let prop = Property::Scalar(5.8);
        assert_eq!(prop.as_scalar(), Some(5.8));
        assert!(prop.as_vector().is_none());
    }

    #[test]
    fn test_vector_property() {
        let prop = Property::Vector(vec![1.0, 2.0, 3.0]);
        assert_eq!(prop.as_vector(), Some(&vec![1.0, 2.0, 3.0]));
        assert!(prop.as_scalar().is_none());
    }
}
