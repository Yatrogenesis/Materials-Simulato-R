//! Query command implementation

use materials_core::Result;

pub async fn execute(_formula: Option<String>) -> Result<()> {
    println!("Query materials from database");
    // TODO: Implement database query
    Ok(())
}
