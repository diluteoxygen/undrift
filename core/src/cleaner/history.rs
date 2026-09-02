use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{OpenOptions, create_dir_all};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRecord {
    pub timestamp: DateTime<Utc>,
    pub items_count: usize,
    pub reclaimed_bytes: u64,
    pub human_reclaimed: String,
    pub permanent: bool,
    pub paths: Vec<String>,
}

pub struct HistoryManager;

impl HistoryManager {
    pub fn history_file_path() -> PathBuf {
        let base = if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            PathBuf::from(local_app_data).join("Undrift")
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".undrift")
        } else {
            PathBuf::from(".undrift")
        };
        base.join("history.jsonl")
    }

    pub fn record_cleanup(
        items_count: usize,
        reclaimed_bytes: u64,
        permanent: bool,
        paths: Vec<String>,
    ) -> std::io::Result<()> {
        let path = Self::history_file_path();
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }

        let record = HistoryRecord {
            timestamp: Utc::now(),
            items_count,
            reclaimed_bytes,
            human_reclaimed: crate::model::candidate::format_size(reclaimed_bytes),
            permanent,
            paths,
        };

        let json = serde_json::to_string(&record)?;
        let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
        writeln!(file, "{json}")?;
        Ok(())
    }

    pub fn load_history() -> Vec<HistoryRecord> {
        let path = Self::history_file_path();
        if !path.exists() {
            return Vec::new();
        }

        let file = match OpenOptions::new().read(true).open(&path) {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };

        let reader = BufReader::new(file);
        reader
            .lines()
            .map_while(Result::ok)
            .filter_map(|line| serde_json::from_str::<HistoryRecord>(&line).ok())
            .collect()
    }
}
