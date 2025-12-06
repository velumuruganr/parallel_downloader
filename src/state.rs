use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Chunk {
    pub start: u64,
    pub end: u64,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadState {
    pub url: String,
    pub chunks: Vec<Chunk>,
}

pub async fn save_state(state: &DownloadState, filename: &str) -> Result<()> {
    let json = serde_json::to_string_pretty(state)?;
    tokio::fs::write(filename, json).await?;
    Ok(())
}
