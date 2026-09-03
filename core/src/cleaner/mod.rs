pub mod history;

use chrono::{DateTime, Utc};
use history::HistoryManager;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanItemSuccess {
    pub path: PathBuf,
    pub bytes_reclaimed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanItemFailure {
    pub path: PathBuf,
    pub error_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanReport {
    pub total_reclaimed_bytes: u64,
    pub human_total_reclaimed: String,
    pub succeeded: Vec<CleanItemSuccess>,
    pub failed: Vec<CleanItemFailure>,
    pub is_dry_run: bool,
    pub was_permanent: bool,
    pub timestamp: DateTime<Utc>,
}

pub struct CleanExecutor;

impl CleanExecutor {
    pub fn clean_targets<P: AsRef<Path>>(
        targets: &[(P, u64)], // (path, size_bytes)
        permanent: bool,
        dry_run: bool,
    ) -> CleanReport {
        let mut succeeded = Vec::new();
        let mut failed = Vec::new();
        let mut total_reclaimed = 0u64;

        for (target, size) in targets {
            let path = target.as_ref();
            if !path.exists() {
                failed.push(CleanItemFailure {
                    path: path.to_path_buf(),
                    error_message: "Path does not exist".to_string(),
                });
                continue;
            }

            if dry_run {
                total_reclaimed += *size;
                succeeded.push(CleanItemSuccess {
                    path: path.to_path_buf(),
                    bytes_reclaimed: *size,
                });
                continue;
            }

            if let Some(reason) = crate::safety::SafetyPipeline::pre_clean_check(path) {
                failed.push(CleanItemFailure {
                    path: path.to_path_buf(),
                    error_message: format!("Safety re-check failed: {}", reason),
                });
                continue;
            }

            let result = if permanent {
                if path.is_dir() {
                    std::fs::remove_dir_all(path)
                } else {
                    std::fs::remove_file(path)
                }
            } else {
                trash::delete(path).map_err(|e| std::io::Error::other(e.to_string()))
            };

            match result {
                Ok(()) => {
                    total_reclaimed += *size;
                    succeeded.push(CleanItemSuccess {
                        path: path.to_path_buf(),
                        bytes_reclaimed: *size,
                    });
                }
                Err(e) => {
                    failed.push(CleanItemFailure {
                        path: path.to_path_buf(),
                        error_message: e.to_string(),
                    });
                }
            }
        }

        if !dry_run && !succeeded.is_empty() {
            let paths_str: Vec<String> = succeeded
                .iter()
                .map(|item| item.path.to_string_lossy().to_string())
                .collect();
            let _ = HistoryManager::record_cleanup(
                succeeded.len(),
                total_reclaimed,
                permanent,
                paths_str,
            );
        }

        CleanReport {
            total_reclaimed_bytes: total_reclaimed,
            human_total_reclaimed: crate::model::candidate::format_size(total_reclaimed),
            succeeded,
            failed,
            is_dry_run: dry_run,
            was_permanent: permanent,
            timestamp: Utc::now(),
        }
    }
}
