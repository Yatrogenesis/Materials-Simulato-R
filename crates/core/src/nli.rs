//! Natural Language Interface for Materials Science
//!
//! Process natural language queries and translate them to LIRS code or database queries.
//! Uses intent recognition, entity extraction, and semantic understanding.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Natural language query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NLQuery {
    pub text: String,
    pub language: Language,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Language {
    English,
    Spanish,
    Chinese,
    Auto,
}

/// Query intent classification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueryIntent {
    /// Search for materials
    Search(SearchIntent),
    /// Predict property
    Predict(PredictIntent),
    /// Discover new materials
    Discover(DiscoverIntent),
    /// Compare materials
    Compare(CompareIntent),
    /// Generate LIRS code
    Generate(GenerateIntent),
    /// Explain concept
    Explain(ExplainIntent),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchIntent {
    pub material_type: Option<String>,
    pub elements: Vec<String>,
    pub properties: HashMap<String, PropertyConstraint>,
    pub application: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PredictIntent {
    pub formula: String,
    pub property: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscoverIntent {
    pub target_property: String,
    pub target_value: f64,
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompareIntent {
    pub materials: Vec<String>,
    pub aspects: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerateIntent {
    pub structure_type: String,
    pub elements: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExplainIntent {
    pub concept: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropertyConstraint {
    pub operator: ComparisonOp,
    pub value: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ComparisonOp {
    GreaterThan,
    LessThan,
    Equal,
    Between,
}

/// Extracted entities from text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntities {
    pub elements: Vec<String>,
    pub properties: Vec<String>,
    pub values: Vec<f64>,
    pub structure_types: Vec<String>,
    pub applications: Vec<String>,
}

/// NLI Processing Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NLIResult {
    pub intent: QueryIntent,
    pub entities: ExtractedEntities,
    pub lirs_code: Option<String>,
    pub confidence: f64,
    pub explanation: String,
}

/// Natural Language Interface Engine
pub struct NLIEngine {
    element_patterns: HashMap<String, Vec<String>>,
    property_patterns: HashMap<String, Vec<String>>,
    structure_patterns: HashMap<String, Vec<String>>,
}

impl NLIEngine {
    pub fn new() -> Self {
        Self {
            element_patterns: Self::init_element_patterns(),
            property_patterns: Self::init_property_patterns(),
            structure_patterns: Self::init_structure_patterns(),
        }
    }

    /// Process natural language query
    pub fn process(&self, query: &NLQuery) -> NLIResult {
        let text = query.text.to_lowercase();

        // Extract entities
        let entities = self.extract_entities(&text);

        // Classify intent
        let intent = self.classify_intent(&text, &entities);

        // Generate LIRS code if applicable
        let lirs_code = self.generate_lirs(&intent, &entities);

        // Calculate confidence
        let confidence = self.calculate_confidence(&text, &intent, &entities);

        // Generate explanation
        let explanation = self.generate_explanation(&intent, &entities);

        NLIResult {
            intent,
            entities,
            lirs_code,
            confidence,
            explanation,
        }
    }

    /// Extract entities from text
    fn extract_entities(&self, text: &str) -> ExtractedEntities {
        let mut elements = Vec::new();
        let mut properties = Vec::new();
        let mut values = Vec::new();
        let mut structure_types = Vec::new();
        let mut applications = Vec::new();

        // Extract elements
        for (element, patterns) in &self.element_patterns {
            for pattern in patterns {
                if text.contains(pattern) {
                    elements.push(element.clone());
                    break;
                }
            }
        }

        // Extract properties
        for (property, patterns) in &self.property_patterns {
            for pattern in patterns {
                if text.contains(pattern) {
                    properties.push(property.clone());
                    break;
                }
            }
        }

        // Extract numeric values
        let words: Vec<&str> = text.split_whitespace().collect();
        for word in words {
            if let Ok(value) = word.parse::<f64>() {
                values.push(value);
            }
        }

        // Extract structure types
        for (structure, patterns) in &self.structure_patterns {
            for pattern in patterns {
                if text.contains(pattern) {
                    structure_types.push(structure.clone());
                    break;
                }
            }
        }

        // Extract applications
        let app_keywords = vec![
            ("battery", "battery cathode"),
            ("solar", "solar cell photovoltaic"),
            ("catalyst", "catalyst catalytic"),
            ("superconductor", "superconductor superconducting"),
            ("sensor", "sensor sensing"),
        ];

        for (app, keywords) in app_keywords {
            if keywords.split_whitespace().any(|kw| text.contains(kw)) {
                applications.push(app.to_string());
            }
        }

        ExtractedEntities {
            elements,
            properties,
            values,
            structure_types,
            applications,
        }
    }

    /// Classify query intent
    fn classify_intent(&self, text: &str, entities: &ExtractedEntities) -> QueryIntent {
        // Search intent
        if text.contains("find") || text.contains("search") || text.contains("show me") {
            return QueryIntent::Search(SearchIntent {
                material_type: entities.structure_types.first().cloned(),
                elements: entities.elements.clone(),
                properties: HashMap::new(),
                application: entities.applications.first().cloned(),
            });
        }

        // Predict intent
        if text.contains("predict") || text.contains("what is the") || text.contains("calculate") {
            if !entities.properties.is_empty() {
                return QueryIntent::Predict(PredictIntent {
                    formula: entities.elements.join(""),
                    property: entities.properties[0].clone(),
                });
            }
        }

        // Discover intent
        if text.contains("discover") || text.contains("design") || text.contains("new material") {
            if !entities.properties.is_empty() && !entities.values.is_empty() {
                return QueryIntent::Discover(DiscoverIntent {
                    target_property: entities.properties[0].clone(),
                    target_value: entities.values[0],
                    constraints: Vec::new(),
                });
            }
        }

        // Compare intent
        if text.contains("compare") || text.contains("difference") || text.contains("vs") {
            return QueryIntent::Compare(CompareIntent {
                materials: entities.elements.chunks(2).map(|chunk| chunk.join("")).collect(),
                aspects: entities.properties.clone(),
            });
        }

        // Generate intent
        if text.contains("generate") || text.contains("create") || text.contains("make") {
            if !entities.structure_types.is_empty() {
                return QueryIntent::Generate(GenerateIntent {
                    structure_type: entities.structure_types[0].clone(),
                    elements: entities.elements.clone(),
                });
            }
        }

        // Explain intent
        if text.contains("what is") || text.contains("explain") || text.contains("define") {
            let concept = entities.structure_types.first()
                .or(entities.properties.first())
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());

            return QueryIntent::Explain(ExplainIntent { concept });
        }

        // Default: search
        QueryIntent::Search(SearchIntent {
            material_type: None,
            elements: entities.elements.clone(),
            properties: HashMap::new(),
            application: None,
        })
    }

    /// Generate LIRS code from intent
    fn generate_lirs(&self, intent: &QueryIntent, entities: &ExtractedEntities) -> Option<String> {
        match intent {
            QueryIntent::Generate(gen) => {
                let structure = &gen.structure_type;
                let elements = &gen.elements;

                let code = match structure.as_str() {
                    "perovskite" if elements.len() >= 3 => {
                        format!("(perovskite :{} :{} :{})", elements[0], elements[1], elements[2])
                    }
                    "spinel" if elements.len() >= 2 => {
                        format!("(spinel :{} :{})", elements[0], elements[1])
                    }
                    "binary-oxide" if !elements.is_empty() => {
                        format!("(binary-oxide :{})", elements[0])
                    }
                    "rutile" if !elements.is_empty() => {
                        format!("(rutile :{})", elements[0])
                    }
                    "fluorite" if elements.len() >= 2 => {
                        format!("(fluorite :{} :{})", elements[0], elements[1])
                    }
                    "lco" => "(lco)".to_string(),
                    "lfp" => "(lfp)".to_string(),
                    "graphene" => "(graphene)".to_string(),
                    _ => return None,
                };

                Some(code)
            }
            QueryIntent::Search(search) => {
                if let Some(structure) = &search.material_type {
                    if !search.elements.is_empty() {
                        let code = format!("({} {})", structure,
                            search.elements.iter().map(|e| format!(":{}", e)).collect::<Vec<_>>().join(" ")
                        );
                        return Some(code);
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Calculate confidence score
    fn calculate_confidence(&self, text: &str, intent: &QueryIntent, entities: &ExtractedEntities) -> f64 {
        let mut score: f64 = 0.5; // Base score

        // Boost if we have clear keywords
        let has_clear_intent = text.contains("find") || text.contains("predict")
            || text.contains("discover") || text.contains("compare")
            || text.contains("generate");

        if has_clear_intent {
            score += 0.2;
        }

        // Boost if we extracted entities
        if !entities.elements.is_empty() {
            score += 0.1;
        }
        if !entities.properties.is_empty() {
            score += 0.1;
        }
        if !entities.structure_types.is_empty() {
            score += 0.1;
        }

        // Boost if we can generate LIRS
        if matches!(intent, QueryIntent::Generate(_)) {
            score += 0.1;
        }

        score.min(1.0_f64)
    }

    /// Generate human-readable explanation
    fn generate_explanation(&self, intent: &QueryIntent, _entities: &ExtractedEntities) -> String {
        match intent {
            QueryIntent::Search(s) => {
                format!("Searching for materials of type {:?} with elements {:?}",
                    s.material_type, s.elements)
            }
            QueryIntent::Predict(p) => {
                format!("Predicting {} for material {}", p.property, p.formula)
            }
            QueryIntent::Discover(d) => {
                format!("Discovering materials with {} = {}", d.target_property, d.target_value)
            }
            QueryIntent::Compare(c) => {
                format!("Comparing materials: {:?} on aspects {:?}", c.materials, c.aspects)
            }
            QueryIntent::Generate(g) => {
                format!("Generating {} structure with elements {:?}", g.structure_type, g.elements)
            }
            QueryIntent::Explain(e) => {
                format!("Explaining concept: {}", e.concept)
            }
        }
    }

    // Initialize patterns
    fn init_element_patterns() -> HashMap<String, Vec<String>> {
        let mut patterns = HashMap::new();

        patterns.insert("Li".to_string(), vec!["lithium".to_string(), "li".to_string()]);
        patterns.insert("Na".to_string(), vec!["sodium".to_string(), "na".to_string()]);
        patterns.insert("K".to_string(), vec!["potassium".to_string()]);
        patterns.insert("Fe".to_string(), vec!["iron".to_string(), "fe".to_string(), "ferro".to_string()]);
        patterns.insert("Co".to_string(), vec!["cobalt".to_string(), "co".to_string()]);
        patterns.insert("Ni".to_string(), vec!["nickel".to_string(), "ni".to_string()]);
        patterns.insert("Mn".to_string(), vec!["manganese".to_string(), "mn".to_string()]);
        patterns.insert("Cu".to_string(), vec!["copper".to_string(), "cu".to_string()]);
        patterns.insert("Ag".to_string(), vec!["silver".to_string(), "ag".to_string()]);
        patterns.insert("Au".to_string(), vec!["gold".to_string(), "au".to_string()]);
        patterns.insert("Ti".to_string(), vec!["titanium".to_string(), "ti".to_string()]);
        patterns.insert("Al".to_string(), vec!["aluminum".to_string(), "aluminium".to_string(), "al".to_string()]);
        patterns.insert("O".to_string(), vec!["oxygen".to_string(), "oxide".to_string()]);
        patterns.insert("C".to_string(), vec!["carbon".to_string()]);
        patterns.insert("N".to_string(), vec!["nitrogen".to_string()]);
        patterns.insert("H".to_string(), vec!["hydrogen".to_string()]);
        patterns.insert("S".to_string(), vec!["sulfur".to_string(), "sulphur".to_string()]);
        patterns.insert("P".to_string(), vec!["phosphorus".to_string(), "phosphate".to_string()]);

        patterns
    }

    fn init_property_patterns() -> HashMap<String, Vec<String>> {
        let mut patterns = HashMap::new();

        patterns.insert("band_gap".to_string(), vec!["band gap".to_string(), "bandgap".to_string()]);
        patterns.insert("formation_energy".to_string(), vec!["formation energy".to_string(), "stability".to_string()]);
        patterns.insert("conductivity".to_string(), vec!["conductivity".to_string(), "conductive".to_string()]);
        patterns.insert("thermal_conductivity".to_string(), vec!["thermal conductivity".to_string()]);
        patterns.insert("electrical_conductivity".to_string(), vec!["electrical conductivity".to_string()]);
        patterns.insert("magnetic_moment".to_string(), vec!["magnetic".to_string(), "magnetism".to_string()]);
        patterns.insert("density".to_string(), vec!["density".to_string(), "mass".to_string()]);
        patterns.insert("melting_point".to_string(), vec!["melting point".to_string(), "melting temperature".to_string()]);
        patterns.insert("hardness".to_string(), vec!["hardness".to_string(), "hard".to_string()]);

        patterns
    }

    fn init_structure_patterns() -> HashMap<String, Vec<String>> {
        let mut patterns = HashMap::new();

        patterns.insert("perovskite".to_string(), vec!["perovskite".to_string()]);
        patterns.insert("spinel".to_string(), vec!["spinel".to_string()]);
        patterns.insert("rutile".to_string(), vec!["rutile".to_string()]);
        patterns.insert("fluorite".to_string(), vec!["fluorite".to_string()]);
        patterns.insert("wurtzite".to_string(), vec!["wurtzite".to_string()]);
        patterns.insert("rocksalt".to_string(), vec!["rock salt".to_string(), "rocksalt".to_string(), "halite".to_string()]);
        patterns.insert("graphene".to_string(), vec!["graphene".to_string()]);
        patterns.insert("layered".to_string(), vec!["layered".to_string(), "layer".to_string()]);

        patterns
    }
}

impl Default for NLIEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_intent() {
        let nli = NLIEngine::new();
        let query = NLQuery {
            text: "Find perovskite materials with lithium and titanium".to_string(),
            language: Language::English,
        };

        let result = nli.process(&query);

        assert!(matches!(result.intent, QueryIntent::Search(_)));
        assert!(result.entities.elements.contains(&"Li".to_string()));
        assert!(result.entities.elements.contains(&"Ti".to_string()));
        assert!(result.entities.structure_types.contains(&"perovskite".to_string()));
    }

    #[test]
    fn test_generate_intent() {
        let nli = NLIEngine::new();
        let query = NLQuery {
            text: "Generate a perovskite with calcium titanium and oxygen".to_string(),
            language: Language::English,
        };

        let result = nli.process(&query);

        assert!(matches!(result.intent, QueryIntent::Generate(_)));
        assert!(result.lirs_code.is_some());
    }

    #[test]
    fn test_predict_intent() {
        let nli = NLIEngine::new();
        let query = NLQuery {
            text: "Predict the band gap of iron oxide".to_string(),
            language: Language::English,
        };

        let result = nli.process(&query);

        assert!(matches!(result.intent, QueryIntent::Predict(_)));
        assert!(result.entities.properties.contains(&"band_gap".to_string()));
    }
}
