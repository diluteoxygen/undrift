use super::ClassificationRule;
use crate::model::category::ArtifactCategory;
use crate::model::file_record::FileRecord;
use crate::scanner::ScanIndex;
use chrono::{Duration, Utc};

pub struct DownloadsRule {
    pub stale_days: i64,
}

impl Default for DownloadsRule {
    fn default() -> Self {
        Self { stale_days: 30 }
    }
}

impl DownloadsRule {
    pub fn new(stale_days: i64) -> Self {
        Self { stale_days }
    }
}

impl ClassificationRule for DownloadsRule {
    fn name(&self) -> &'static str {
        "Stale Downloaded Installers"
    }

    fn evaluate(
        &self,
        _index: &ScanIndex,
        record: &FileRecord,
    ) -> Option<(ArtifactCategory, String)> {
        if record.is_dir {
            return None;
        }

        let in_downloads = if !record.path.as_os_str().is_empty() {
            let path_str = record.path.to_string_lossy();
            path_str.contains("/Downloads/")
                || path_str.contains(r"\Downloads\")
                || path_str.ends_with("/Downloads")
                || path_str.ends_with(r"\Downloads")
        } else {
            let mut curr = record.parent_id;
            let mut found = false;
            for _ in 0..5 {
                if let Some(parent) = _index.get_record(curr) {
                    if parent.name.eq_ignore_ascii_case("downloads") {
                        found = true;
                        break;
                    }
                    curr = parent.parent_id;
                } else {
                    break;
                }
            }
            found
        };

        if !in_downloads {
            return None;
        }

        let ext = std::path::Path::new(&record.name)
            .extension()
            .and_then(|s| s.to_str())?
            .to_lowercase();
        let is_installer = matches!(ext.as_str(), "exe" | "msi" | "iso" | "dmg" | "pkg");

        if !is_installer {
            return None;
        }

        let now = Utc::now();
        if let Some(modified) = record.modified_at {
            let age = now.signed_duration_since(modified);
            if age > Duration::days(self.stale_days) {
                let days = age.num_days();
                return Some((
                    ArtifactCategory::StaleInstaller,
                    format!(
                        "Downloaded installer older than {} days ({} days old)",
                        self.stale_days, days
                    ),
                ));
            }
        }

        None
    }
}
