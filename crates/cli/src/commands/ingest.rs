//! Ingest command - Massive data ingestion from worldwide sources

use materials_database::{
    etl_pipeline::{ETLPipeline, MaterialsProjectAdapter, PubChemAdapter, SourceAdapter, DataSource},
    postgres::PostgresDatabase,
};
use std::sync::Arc;
use tracing::info;

pub async fn run(limit: Option<u64>, source: &str) -> anyhow::Result<()> {
    info!("🌍 Materials-Simulato-R - Massive Data Ingestion");
    info!("================================================\n");

    // Connect to database
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/materials".to_string());

    info!("Connecting to database...");
    let db = Arc::new(PostgresDatabase::new(&db_url).await?) as Arc<dyn materials_database::MaterialDatabase>;
    info!("✅ Database connected\n");

    // Create ETL pipeline
    let pipeline = Arc::new(ETLPipeline::new(db, 10)); // 10 parallel workers

    // Determine fetch limit based on user input
    let fetch_limit = limit.or(Some(5000)); // Default to 5000 if not specified

    info!("Configuration:");
    info!("   Sources: {}", source);
    info!("   Limit per source: {:?}\n", fetch_limit);

    let should_ingest_mp = source == "all" || source == "materials-project";
    let should_ingest_pubchem = source == "all" || source == "pubchem";

    let mut mp_result = None;
    let mut pubchem_result = None;

    // Ingest from Materials Project
    if should_ingest_mp {
        info!("📦 Source 1: Materials Project");
        info!("   Estimated: ~150,000 materials");
        let mp_adapter = MaterialsProjectAdapter::new(None);
        let mp_data = mp_adapter.fetch(fetch_limit).await?;

        let estimate = pipeline.estimate_time(mp_data.len() as u64);
        info!("   Estimated time: {:.1} seconds\n", estimate.as_secs_f64());

        let result = pipeline.ingest(DataSource::MaterialsProject, mp_data).await?;

        info!("\n📊 Materials Project Results:");
        info!("   Total: {}", result.total_records);
        info!("   ✅ Success: {}", result.successful);
        info!("   ❌ Failed: {}", result.failed);
        info!("   🔄 Duplicates: {}", result.duplicates);
        info!("   ⚠️  Validation errors: {}", result.validation_errors);
        info!("   ⏱️  Time: {:.2}s", result.elapsed_seconds);
        info!("   📈 Rate: {:.0} records/s\n",
              result.total_records as f64 / result.elapsed_seconds);

        mp_result = Some(result);
    }

    // Ingest from PubChem
    if should_ingest_pubchem {
        info!("📦 Source 2: PubChem");
        info!("   Estimated: ~100,000,000 compounds");
        let pubchem_adapter = PubChemAdapter;
        let pubchem_data = pubchem_adapter.fetch(fetch_limit).await?;

        let estimate = pipeline.estimate_time(pubchem_data.len() as u64);
        info!("   Estimated time: {:.1} seconds\n", estimate.as_secs_f64());

        let result = pipeline.ingest(DataSource::PubChem, pubchem_data).await?;

        info!("\n📊 PubChem Results:");
        info!("   Total: {}", result.total_records);
        info!("   ✅ Success: {}", result.successful);
        info!("   ❌ Failed: {}", result.failed);
        info!("   🔄 Duplicates: {}", result.duplicates);
        info!("   ⚠️  Validation errors: {}", result.validation_errors);
        info!("   ⏱️  Time: {:.2}s", result.elapsed_seconds);
        info!("   📈 Rate: {:.0} records/s\n",
              result.total_records as f64 / result.elapsed_seconds);

        pubchem_result = Some(result);
    }

    // Summary
    if mp_result.is_some() || pubchem_result.is_some() {
        let total_ingested = mp_result.as_ref().map(|r| r.successful).unwrap_or(0)
            + pubchem_result.as_ref().map(|r| r.successful).unwrap_or(0);
        let total_duplicates = mp_result.as_ref().map(|r| r.duplicates).unwrap_or(0)
            + pubchem_result.as_ref().map(|r| r.duplicates).unwrap_or(0);
        let total_records = mp_result.as_ref().map(|r| r.total_records).unwrap_or(0)
            + pubchem_result.as_ref().map(|r| r.total_records).unwrap_or(0);
        let total_time = mp_result.as_ref().map(|r| r.elapsed_seconds).unwrap_or(0.0)
            + pubchem_result.as_ref().map(|r| r.elapsed_seconds).unwrap_or(0.0);

        info!("\n═══════════════════════════════════════");
        info!("  📊 GLOBAL INGESTION SUMMARY");
        info!("═══════════════════════════════════════");
        info!("Total materials ingested: {} ✅", total_ingested);
        info!("Total duplicates skipped: {} 🔄", total_duplicates);
        info!("Total time: {:.2}s ⏱️", total_time);
        if total_time > 0.0 {
            info!("Average rate: {:.0} records/s 📈",
                  total_records as f64 / total_time);
        }
        info!("═══════════════════════════════════════\n");

        info!("✨ Ingestion complete! Database populated with worldwide data.");
    } else {
        info!("⚠️  No sources were selected for ingestion.");
        info!("   Available sources: all, materials-project, pubchem");
    }

    Ok(())
}
