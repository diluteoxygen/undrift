use super::category::ArtifactCategory;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum GitRepoStatus {
    NotInRepo,
    Clean,
    Dirty { modified_count: usize },
    Error { message: String },
}

impl GitRepoStatus {
    pub fn is_safe(&self) -> bool {
        match self {
            Self::NotInRepo | Self::Clean => true,
            Self::Dirty { .. } | Self::Error { .. } => false,
        }
    }

    pub fn display_summary(&self) -> String {
        match self {
            Self::NotInRepo => "Not in Git repository".to_string(),
            Self::Clean => "Git working tree clean".to_string(),
            Self::Dirty { modified_count } => {
                format!("Git working tree dirty ({modified_count} uncommitted files)")
            }
            Self::Error { message } => format!("Git check error: {message}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReclaimCandidate {
    pub id: String,
    pub category: ArtifactCategory,
    pub path: PathBuf,
    pub display_path: String,
    pub size_bytes: u64,
    pub human_size: String,
    pub last_modified: Option<DateTime<Utc>>,
    pub file_count: u64,
    pub is_safe: bool,
    pub safety_reason: String,
    pub git_status: GitRepoStatus,
    pub default_selected: bool,
    pub hardlink_shared_bytes: u64,
    pub has_hardlinks: bool,
    pub size_caveat: Option<String>,
}

impl ReclaimCandidate {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        category: ArtifactCategory,
        path: PathBuf,
        size_bytes: u64,
        last_modified: Option<DateTime<Utc>>,
        file_count: u64,
        is_safe: bool,
        safety_reason: String,
        git_status: GitRepoStatus,
    ) -> Self {
        let display_path = path.to_string_lossy().to_string();
        let human_size = format_size(size_bytes);
        let default_selected = is_safe;

        Self {
            id: Uuid::new_v4().to_string(),
            category,
            path,
            display_path,
            size_bytes,
            human_size,
            last_modified,
            file_count,
            is_safe,
            safety_reason,
            git_status,
            default_selected,
            hardlink_shared_bytes: 0,
            has_hardlinks: false,
            size_caveat: None,
        }
    }

    pub fn with_hardlink_info(mut self, shared_bytes: u64) -> Self {
        if shared_bytes > 0 {
            self.hardlink_shared_bytes = shared_bytes;
            self.has_hardlinks = true;
            self.size_caveat = Some(format!(
                "Contains hardlinked files sharing {} with other locations (e.g. pnpm store); actual freed disk space may be less than logical size.",
                format_size(shared_bytes)
            ));
        }
        self
    }
}

pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}
