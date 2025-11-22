# STEM Bibliography APIs Integration

## 📚 Overview

Materials-Simulato-R integrates with the most important STEM scientific databases and APIs to provide automated literature ingestion and discovery capabilities.

## 🌐 Supported APIs

### 1. **arXiv**
- **URL**: https://arxiv.org/help/api/index
- **Coverage**: Physics, Mathematics, Computer Science, Materials Science
- **API Key**: Not required
- **Rate Limit**: 1 request per 3 seconds
- **Categories**:
  - `cond-mat.mtrl-sci` - Materials Science
  - `cond-mat.supr-con` - Superconductivity
  - `physics.chem-ph` - Chemical Physics

### 2. **PubMed/PMC E-utilities**
- **URL**: https://www.ncbi.nlm.nih.gov/books/NBK25501/
- **Coverage**: Biomedicine, Biomaterials, Medical Materials
- **API Key**: Optional (recommended) - Get from https://www.ncbi.nlm.nih.gov/account/
- **Rate Limit**: 3 requests/second without key, 10 requests/second with key
- **Email**: Required (NCBI policy)

### 3. **Semantic Scholar**
- **URL**: https://api.semanticscholar.org/api-docs/
- **Coverage**: AI-powered cross-disciplinary search
- **API Key**: Optional - Get from https://www.semanticscholar.org/product/api
- **Rate Limit**: 100 requests/5 minutes without key, 5000 requests/5 minutes with key
- **Features**: Citation graphs, influential papers, AI recommendations

### 4. **OpenAlex**
- **URL**: https://docs.openalex.org/
- **Coverage**: Comprehensive scholarly knowledge graph (replacement for Microsoft Academic)
- **API Key**: Not required
- **Rate Limit**: 100,000 requests/day (polite pool), 10 requests/second
- **Email**: Required for polite pool access

### 5. **Materials Project**
- **URL**: https://next-gen.materialsproject.org/api
- **Coverage**: Computational materials science database
- **API Key**: Required - Get from https://next-gen.materialsproject.org/api
- **Rate Limit**: Varies by plan
- **Data**: Crystal structures, energy calculations, properties

### 6. **NREL Materials Database**
- **URL**: https://developer.nrel.gov/docs/materials/
- **Coverage**: Clean energy materials
- **API Key**: Required - Get from https://developer.nrel.gov/signup/
- **Focus**: Photovoltaics, batteries, thermoelectrics

### 7. **NOMAD Repository**
- **URL**: https://nomad-lab.eu/prod/v1/api/v1/extensions/docs
- **Coverage**: Novel Materials Discovery Laboratory
- **API Key**: Not required
- **Data**: DFT calculations, experimental data, AI predictions

---

## 🔑 Environment Variables

Create a `.env` file with your API keys:

```bash
# Semantic Scholar (optional but recommended)
SEMANTIC_SCHOLAR_API_KEY=your_key_here

# PubMed E-utilities
PUBMED_API_KEY=your_key_here
PUBMED_EMAIL=your.email@example.com

# OpenAlex
OPENALEX_EMAIL=your.email@example.com

# Materials Project
MATERIALS_PROJECT_API_KEY=your_key_here

# NREL
NREL_API_KEY=your_key_here
```

---

## 📖 Usage Examples

### Basic Search

```rust
use materials_database::stem_bibliography_apis::STEMBibliography;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize with environment variables
    let bib = STEMBibliography::new().with_env_keys();

    // Search all sources in parallel
    let papers = bib.search_all("perovskite solar cells").await?;

    println!("Found {} papers", papers.len());
    for paper in papers.iter().take(5) {
        println!("  - {} ({})", paper.title, paper.source);
    }

    Ok(())
}
```

### Source-Specific Search

```rust
// Search only arXiv
let arxiv_papers = bib.search_arxiv("topological insulator").await?;

// Search only PubMed
let pubmed_papers = bib.search_pubmed("bone implant materials").await?;

// Search Semantic Scholar (AI-powered)
let ss_papers = bib.search_semantic_scholar("lithium battery cathode").await?;

// Search OpenAlex (comprehensive)
let openalex_papers = bib.search_openalex("superconductor").await?;

// Search Materials Project
let mp_data = bib.search_materials_project("LiFePO4").await?;
```

### Pre-configured Queries

```rust
use materials_database::stem_bibliography_apis::queries;

// Use predefined queries for common topics
let perovskite_papers = bib.search_all(queries::PEROVSKITES).await?;
let battery_papers = bib.search_all(queries::BATTERIES).await?;
let superconductor_papers = bib.search_all(queries::SUPERCONDUCTORS).await?;
let thermoelectric_papers = bib.search_all(queries::THERMOELECTRICS).await?;
let topological_papers = bib.search_all(queries::TOPOLOGICAL).await?;
let 2d_material_papers = bib.search_all(queries::2D_MATERIALS).await?;
let wide_bandgap_papers = bib.search_all(queries::WIDE_BANDGAP).await?;
let magnetic_papers = bib.search_all(queries::MAGNETIC).await?;
let catalyst_papers = bib.search_all(queries::CATALYSTS).await?;
let biomaterial_papers = bib.search_all(queries::BIOMATERIALS).await?;
```

### Caching and Stats

```rust
// Get cached paper by ID
if let Some(paper) = bib.get_cached("arxiv:2311.12345").await {
    println!("Cached: {}", paper.title);
}

// Get usage statistics
let stats = bib.get_stats().await;
println!("Total papers cached: {}", stats.total_papers_cached);
for (source, count) in stats.papers_by_source {
    println!("  {:?}: {} papers", source, count);
}

// Clear cache
bib.clear_cache().await;
```

---

## 🎯 Advanced Features

### Parallel Multi-Source Search

The `search_all()` method queries all APIs concurrently using `tokio::join!` for maximum performance:

```rust
// This queries arXiv, PubMed, Semantic Scholar, and OpenAlex simultaneously
let all_papers = bib.search_all("graphene oxide").await?;

// Results are automatically deduplicated and cached
```

### Custom arXiv Categories

```rust
// Materials science specific
let query = "cat:cond-mat.mtrl-sci AND ti:perovskite";
let papers = bib.search_arxiv(query).await?;

// Superconductivity
let query = "cat:cond-mat.supr-con AND abs:high-temperature";
let papers = bib.search_arxiv(query).await?;
```

### PubMed MeSH Terms

```rust
// Search using Medical Subject Headings
let query = "biomaterials[MeSH] AND titanium[Title/Abstract]";
let papers = bib.search_pubmed(query).await?;
```

---

## 📊 Data Structure

Each paper returned contains:

```rust
pub struct Paper {
    pub id: String,                      // Unique identifier
    pub title: String,                    // Paper title
    pub authors: Vec<String>,             // Author names
    pub abstract_text: Option<String>,    // Abstract
    pub doi: Option<String>,              // DOI
    pub arxiv_id: Option<String>,         // arXiv ID
    pub pubmed_id: Option<String>,        // PubMed ID
    pub published_date: Option<String>,   // Publication date
    pub journal: Option<String>,          // Journal name
    pub keywords: Vec<String>,            // Keywords/tags
    pub citations: usize,                 // Citation count
    pub source: BibliographySource,       // Source API
    pub url: String,                      // Paper URL
    pub pdf_url: Option<String>,          // PDF download URL
}
```

---

## 🔍 Pre-configured Research Topics

### Available Query Constants

- `queries::PEROVSKITES` - Perovskite photovoltaic materials
- `queries::BATTERIES` - Lithium-ion battery cathode materials
- `queries::SUPERCONDUCTORS` - High-temperature superconductors
- `queries::THERMOELECTRICS` - Thermoelectric materials
- `queries::TOPOLOGICAL` - Topological insulators
- `queries::2D_MATERIALS` - Graphene and 2D materials
- `queries::WIDE_BANDGAP` - Wide bandgap semiconductors (GaN, SiC)
- `queries::MAGNETIC` - Magnetic materials and spintronics
- `queries::CATALYSTS` - Catalytic materials
- `queries::BIOMATERIALS` - Biocompatible materials and implants

---

## ⚡ Performance Tips

1. **Use API Keys**: Significantly increases rate limits
2. **Enable Caching**: Avoid redundant API calls
3. **Batch Queries**: Use `search_all()` for parallel execution
4. **Respect Rate Limits**: Implement exponential backoff if needed
5. **Cache Results**: Papers are automatically cached in memory

---

## 🚀 Future Enhancements

- [ ] Implement actual HTTP requests (currently scaffolded)
- [ ] Add XML parsing for PubMed responses
- [ ] Implement rate limiting middleware
- [ ] Add persistent cache (Redis/PostgreSQL)
- [ ] Implement semantic deduplication
- [ ] Add citation network analysis
- [ ] Implement auto-update of cached papers
- [ ] Add BibTeX export
- [ ] Implement PDF download capability
- [ ] Add full-text search when available

---

## 📚 References

- arXiv API: https://arxiv.org/help/api/
- PubMed E-utilities: https://www.ncbi.nlm.nih.gov/books/NBK25501/
- Semantic Scholar API: https://api.semanticscholar.org/api-docs/
- OpenAlex: https://docs.openalex.org/
- Materials Project: https://next-gen.materialsproject.org/api
- NREL Developer Network: https://developer.nrel.gov/
- NOMAD: https://nomad-lab.eu/

---

**Author**: Francisco Molina Burgos (Yatrogenesis)
**Date**: 2025-11-22
**License**: MIT OR Apache-2.0
