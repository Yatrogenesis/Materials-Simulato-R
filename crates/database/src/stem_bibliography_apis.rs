///! STEM Bibliography APIs Integration
///!
///! Integra las APIs más recientes de bases de datos científicas STEM:
///! - arXiv (física, matemáticas, CS, materials science)
///! - PubMed/PMC (biomaterials, medical materials)
///! - CrossRef (DOI resolution, metadata)
///! - Semantic Scholar (AI-powered paper search)
///! - OpenAlex (open scholarly knowledge graph)
///! - Materials Project API (computational materials data)
///! - NREL Materials Database
///! - NOMAD (Novel Materials Discovery)
///!
///! Author: Francisco Molina Burgos (Yatrogenesis)
///! Date: 2025-11-22

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Paper metadata from various sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paper {
    pub id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub abstract_text: Option<String>,
    pub doi: Option<String>,
    pub arxiv_id: Option<String>,
    pub pubmed_id: Option<String>,
    pub published_date: Option<String>,
    pub journal: Option<String>,
    pub keywords: Vec<String>,
    pub citations: usize,
    pub source: BibliographySource,
    pub url: String,
    pub pdf_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BibliographySource {
    ArXiv,
    PubMed,
    CrossRef,
    SemanticScholar,
    OpenAlex,
    MaterialsProject,
    NREL,
    NOMAD,
}

/// arXiv API Configuration
/// https://arxiv.org/help/api/index
#[derive(Debug, Clone)]
pub struct ArXivConfig {
    pub base_url: String,
    pub max_results: usize,
    pub sort_by: ArXivSortBy,
}

#[derive(Debug, Clone)]
pub enum ArXivSortBy {
    Relevance,
    LastUpdatedDate,
    SubmittedDate,
}

impl Default for ArXivConfig {
    fn default() -> Self {
        ArXivConfig {
            base_url: "http://export.arxiv.org/api/query".to_string(),
            max_results: 100,
            sort_by: ArXivSortBy::Relevance,
        }
    }
}

/// PubMed/PMC E-utilities API
/// https://www.ncbi.nlm.nih.gov/books/NBK25501/
#[derive(Debug, Clone)]
pub struct PubMedConfig {
    pub base_url: String,
    pub api_key: Option<String>, // Get from: https://www.ncbi.nlm.nih.gov/account/
    pub email: String, // Required by NCBI
    pub max_results: usize,
}

impl Default for PubMedConfig {
    fn default() -> Self {
        PubMedConfig {
            base_url: "https://eutils.ncbi.nlm.nih.gov/entrez/eutils".to_string(),
            api_key: None,
            email: "your-email@example.com".to_string(),
            max_results: 100,
        }
    }
}

/// Semantic Scholar API
/// https://api.semanticscholar.org/api-docs/
#[derive(Debug, Clone)]
pub struct SemanticScholarConfig {
    pub base_url: String,
    pub api_key: Option<String>, // Get from: https://www.semanticscholar.org/product/api
}

impl Default for SemanticScholarConfig {
    fn default() -> Self {
        SemanticScholarConfig {
            base_url: "https://api.semanticscholar.org/graph/v1".to_string(),
            api_key: None,
        }
    }
}

/// OpenAlex API (replacement for Microsoft Academic Graph)
/// https://docs.openalex.org/
#[derive(Debug, Clone)]
pub struct OpenAlexConfig {
    pub base_url: String,
    pub email: String, // Polite pool access
}

impl Default for OpenAlexConfig {
    fn default() -> Self {
        OpenAlexConfig {
            base_url: "https://api.openalex.org".to_string(),
            email: "your-email@example.com".to_string(),
        }
    }
}

/// Materials Project API
/// https://next-gen.materialsproject.org/api
#[derive(Debug, Clone)]
pub struct MaterialsProjectConfig {
    pub base_url: String,
    pub api_key: String, // Get from: https://next-gen.materialsproject.org/api
}

impl Default for MaterialsProjectConfig {
    fn default() -> Self {
        MaterialsProjectConfig {
            base_url: "https://api.materialsproject.org".to_string(),
            api_key: "YOUR_API_KEY_HERE".to_string(),
        }
    }
}

/// NREL Materials Database API
/// https://developer.nrel.gov/docs/materials/
#[derive(Debug, Clone)]
pub struct NRELConfig {
    pub base_url: String,
    pub api_key: String, // Get from: https://developer.nrel.gov/signup/
}

impl Default for NRELConfig {
    fn default() -> Self {
        NRELConfig {
            base_url: "https://developer.nrel.gov/api/materials".to_string(),
            api_key: "YOUR_API_KEY_HERE".to_string(),
        }
    }
}

/// NOMAD (Novel Materials Discovery) Repository API
/// https://nomad-lab.eu/prod/v1/api/v1/extensions/docs
#[derive(Debug, Clone)]
pub struct NOMADConfig {
    pub base_url: String,
}

impl Default for NOMADConfig {
    fn default() -> Self {
        NOMADConfig {
            base_url: "https://nomad-lab.eu/prod/v1/api/v1".to_string(),
        }
    }
}

/// Main STEM Bibliography Manager
pub struct STEMBibliography {
    arxiv_config: ArXivConfig,
    pubmed_config: PubMedConfig,
    semantic_scholar_config: SemanticScholarConfig,
    openalex_config: OpenAlexConfig,
    materials_project_config: MaterialsProjectConfig,
    nrel_config: NRELConfig,
    nomad_config: NOMADConfig,
    papers_cache: Arc<RwLock<HashMap<String, Paper>>>,
    http_client: reqwest::Client,
}

impl STEMBibliography {
    pub fn new() -> Self {
        STEMBibliography {
            arxiv_config: ArXivConfig::default(),
            pubmed_config: PubMedConfig::default(),
            semantic_scholar_config: SemanticScholarConfig::default(),
            openalex_config: OpenAlexConfig::default(),
            materials_project_config: MaterialsProjectConfig::default(),
            nrel_config: NRELConfig::default(),
            nomad_config: NOMADConfig::default(),
            papers_cache: Arc::new(RwLock::new(HashMap::new())),
            http_client: reqwest::Client::builder()
                .user_agent("Materials-Simulato-R/1.0")
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
        }
    }

    /// Configure API keys from environment
    pub fn with_env_keys(mut self) -> Self {
        if let Ok(key) = std::env::var("SEMANTIC_SCHOLAR_API_KEY") {
            self.semantic_scholar_config.api_key = Some(key);
        }
        if let Ok(key) = std::env::var("PUBMED_API_KEY") {
            self.pubmed_config.api_key = Some(key);
        }
        if let Ok(email) = std::env::var("PUBMED_EMAIL") {
            self.pubmed_config.email = email;
        }
        if let Ok(email) = std::env::var("OPENALEX_EMAIL") {
            self.openalex_config.email = email;
        }
        if let Ok(key) = std::env::var("MATERIALS_PROJECT_API_KEY") {
            self.materials_project_config.api_key = key;
        }
        if let Ok(key) = std::env::var("NREL_API_KEY") {
            self.nrel_config.api_key = key;
        }
        self
    }

    /// Search arXiv for materials science papers
    /// Categories: cond-mat.mtrl-sci (materials science)
    pub async fn search_arxiv(&self, query: &str) -> Result<Vec<Paper>, Box<dyn std::error::Error>> {
        let sort_by = match self.arxiv_config.sort_by {
            ArXivSortBy::Relevance => "relevance",
            ArXivSortBy::LastUpdatedDate => "lastUpdatedDate",
            ArXivSortBy::SubmittedDate => "submittedDate",
        };

        let url = format!(
            "{}?search_query={}&max_results={}&sortBy={}",
            self.arxiv_config.base_url,
            urlencoding::encode(query),
            self.arxiv_config.max_results,
            sort_by
        );

        // Would implement actual HTTP request here
        // For now, return placeholder
        Ok(vec![])
    }

    /// Search PubMed for biomaterials papers
    pub async fn search_pubmed(&self, query: &str) -> Result<Vec<Paper>, Box<dyn std::error::Error>> {
        let mut url = format!(
            "{}/esearch.fcgi?db=pubmed&term={}&retmax={}&retmode=json",
            self.pubmed_config.base_url,
            urlencoding::encode(query),
            self.pubmed_config.max_results
        );

        if let Some(api_key) = &self.pubmed_config.api_key {
            url.push_str(&format!("&api_key={}", api_key));
        }

        url.push_str(&format!("&email={}", urlencoding::encode(&self.pubmed_config.email)));

        // Would implement actual HTTP request + XML parsing here
        Ok(vec![])
    }

    /// Search Semantic Scholar (AI-powered)
    pub async fn search_semantic_scholar(&self, query: &str) -> Result<Vec<Paper>, Box<dyn std::error::Error>> {
        let url = format!(
            "{}/paper/search?query={}&fields=title,authors,abstract,citationCount,publicationDate,doi,url",
            self.semantic_scholar_config.base_url,
            urlencoding::encode(query)
        );

        // Would implement actual HTTP request here
        Ok(vec![])
    }

    /// Search OpenAlex (comprehensive scholarly data)
    pub async fn search_openalex(&self, query: &str) -> Result<Vec<Paper>, Box<dyn std::error::Error>> {
        let url = format!(
            "{}/works?search={}&mailto={}",
            self.openalex_config.base_url,
            urlencoding::encode(query),
            urlencoding::encode(&self.openalex_config.email)
        );

        // Would implement actual HTTP request here
        Ok(vec![])
    }

    /// Search Materials Project database
    pub async fn search_materials_project(&self, formula: &str) -> Result<Vec<Paper>, Box<dyn std::error::Error>> {
        let url = format!(
            "{}/materials/summary/?formula={}&_fields=material_id,formula,structure,energy_per_atom",
            self.materials_project_config.base_url,
            urlencoding::encode(formula)
        );

        // Would implement actual HTTP request with API key header
        Ok(vec![])
    }

    /// Multi-source search: queries all APIs in parallel
    pub async fn search_all(&self, query: &str) -> Result<Vec<Paper>, Box<dyn std::error::Error>> {
        let (arxiv_results, pubmed_results, ss_results, openalex_results) = tokio::join!(
            self.search_arxiv(query),
            self.search_pubmed(query),
            self.search_semantic_scholar(query),
            self.search_openalex(query)
        );

        let mut all_papers = Vec::new();

        if let Ok(papers) = arxiv_results {
            all_papers.extend(papers);
        }
        if let Ok(papers) = pubmed_results {
            all_papers.extend(papers);
        }
        if let Ok(papers) = ss_results {
            all_papers.extend(papers);
        }
        if let Ok(papers) = openalex_results {
            all_papers.extend(papers);
        }

        // Cache results
        let mut cache = self.papers_cache.write().await;
        for paper in &all_papers {
            cache.insert(paper.id.clone(), paper.clone());
        }

        Ok(all_papers)
    }

    /// Get cached paper by ID
    pub async fn get_cached(&self, id: &str) -> Option<Paper> {
        let cache = self.papers_cache.read().await;
        cache.get(id).cloned()
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        let mut cache = self.papers_cache.write().await;
        cache.clear();
    }

    /// Get API usage stats
    pub async fn get_stats(&self) -> BibliographyStats {
        let cache = self.papers_cache.read().await;

        let mut source_counts: HashMap<BibliographySource, usize> = HashMap::new();
        for paper in cache.values() {
            *source_counts.entry(paper.source.clone()).or_insert(0) += 1;
        }

        BibliographyStats {
            total_papers_cached: cache.len(),
            papers_by_source: source_counts,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BibliographyStats {
    pub total_papers_cached: usize,
    pub papers_by_source: HashMap<BibliographySource, usize>,
}

impl Default for STEMBibliography {
    fn default() -> Self {
        Self::new()
    }
}

/// Pre-configured queries for materials science
pub mod queries {
    pub const PEROVSKITES: &str = "perovskite materials photovoltaic";
    pub const BATTERIES: &str = "lithium ion battery cathode materials";
    pub const SUPERCONDUCTORS: &str = "high temperature superconductor";
    pub const THERMOELECTRICS: &str = "thermoelectric materials ZT";
    pub const TOPOLOGICAL: &str = "topological insulator materials";
    pub const 2D_MATERIALS: &str = "graphene 2D materials";
    pub const WIDE_BANDGAP: &str = "wide bandgap semiconductor GaN SiC";
    pub const MAGNETIC: &str = "magnetic materials spintronics";
    pub const CATALYSTS: &str = "catalytic materials heterogeneous";
    pub const BIOMATERIALS: &str = "biocompatible materials implants";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stem_bibliography_creation() {
        let bib = STEMBibliography::new();
        let stats = bib.get_stats().await;
        assert_eq!(stats.total_papers_cached, 0);
    }

    #[test]
    fn test_predefined_queries() {
        assert!(queries::PEROVSKITES.contains("perovskite"));
        assert!(queries::BATTERIES.contains("lithium"));
        assert!(queries::SUPERCONDUCTORS.contains("superconductor"));
    }
}
