//! Compute Worker Service
//!
//! Distributed compute worker for materials simulations

use materials_compute::ComputationMethod;
use materials_core::Material;
use materials_database::redis_cache::RedisCache;
use materials_monitoring;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, error};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct ComputeJob {
    id: Uuid,
    material_formula: String,
    computation_type: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct ComputationResult {
    energy: Option<f64>,
    forces: Option<Vec<[f64; 3]>>,
    cost_seconds: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct JobResult {
    job_id: Uuid,
    status: JobStatus,
    result: Option<ComputationResult>,
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

/// Simple mock computation method for demonstration
struct MockComputationMethod;

#[async_trait::async_trait]
impl ComputationMethod for MockComputationMethod {
    async fn calculate_energy(&self, material: &Material) -> materials_compute::Result<f64> {
        // Mock energy calculation based on formula length (just for demo)
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(-1.23 * material.formula.len() as f64)
    }

    async fn calculate_forces(&self, _material: &Material) -> materials_compute::Result<Vec<[f64; 3]>> {
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok(vec![[0.1, 0.2, 0.3]])
    }

    fn cost_estimate(&self, _material: &Material) -> f64 {
        0.15 // 150ms
    }

    fn name(&self) -> &str {
        "mock-computation"
    }
}

async fn process_job(job: ComputeJob, method: &dyn ComputationMethod) -> JobResult {
    info!("Processing job {} for material {} using {}",
          job.id, job.material_formula, method.name());

    let material = Material::new(&job.material_formula);

    // Calculate energy and forces
    let energy_result = method.calculate_energy(&material).await;
    let forces_result = method.calculate_forces(&material).await;

    match (energy_result, forces_result) {
        (Ok(energy), Ok(forces)) => {
            info!("Job {} completed successfully", job.id);
            JobResult {
                job_id: job.id,
                status: JobStatus::Completed,
                result: Some(ComputationResult {
                    energy: Some(energy),
                    forces: Some(forces),
                    cost_seconds: method.cost_estimate(&material),
                }),
                error: None,
            }
        }
        (Err(e), _) | (_, Err(e)) => {
            error!("Job {} failed: {}", job.id, e);
            JobResult {
                job_id: job.id,
                status: JobStatus::Failed,
                result: None,
                error: Some(e.to_string()),
            }
        }
    }
}

async fn worker_loop(redis: RedisCache, worker_id: &str) -> anyhow::Result<()> {
    info!("Worker {} started, polling for jobs...", worker_id);

    let computation_method = MockComputationMethod;

    loop {
        // Try to pop a job from the queue
        match redis.get::<ComputeJob>("compute:queue:pending").await {
            Ok(Some(job)) => {
                info!("Worker {} picked up job {}", worker_id, job.id);

                // Mark job as processing
                let processing_key = format!("compute:job:{}:status", job.id);
                let _ = redis.set(&processing_key, &JobStatus::Processing, Some(3600)).await;

                // Process the job
                let result = process_job(job, &computation_method).await;

                // Store result
                let result_key = format!("compute:job:{}:result", result.job_id);
                match redis.set(&result_key, &result, Some(86400)).await {
                    Ok(_) => info!("Stored result for job {}", result.job_id),
                    Err(e) => error!("Failed to store result: {}", e),
                }
            }
            Ok(None) => {
                // No jobs available, wait before polling again
                sleep(Duration::from_secs(5)).await;
            }
            Err(e) => {
                error!("Failed to poll queue: {}", e);
                sleep(Duration::from_secs(10)).await;
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize monitoring
    materials_monitoring::init()?;

    let worker_id = format!("worker-{}", Uuid::new_v4());
    info!("Starting Materials-Simulato-R Compute Worker v{} ({})",
          env!("CARGO_PKG_VERSION"), worker_id);

    // Connect to Redis queue
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());

    info!("Connecting to Redis at {}", redis_url);
    let redis = RedisCache::new(&redis_url).await?;

    info!("Redis connection established");

    // Start worker loop
    let worker_handle = tokio::spawn({
        let worker_id = worker_id.clone();
        async move {
            if let Err(e) = worker_loop(redis, &worker_id).await {
                error!("Worker loop failed: {}", e);
            }
        }
    });

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    info!("Shutting down compute worker {}", worker_id);

    // Allow graceful shutdown
    worker_handle.abort();

    Ok(())
}
