use super::ClassificationRule;
use crate::model::category::ArtifactCategory;
use crate::model::file_record::FileRecord;
use crate::scanner::ScanIndex;

pub struct WindowsUpdateRule;

impl ClassificationRule for WindowsUpdateRule {
    fn name(&self) -> &'static str {
        "Windows Update Leftovers"
    }

    fn evaluate(
        &self,
        _index: &ScanIndex,
        record: &FileRecord,
    ) -> Option<(ArtifactCategory, String)> {
        if !record.is_dir {
            return None;
        }

        let name = record.name.to_lowercase();
        if name == "windows.old" {
            return Some((
                ArtifactCategory::WindowsUpdate,
                "Backup archive from prior Windows version (Windows.old)".to_string(),
            ));
        }

        let in_softwaredistribution = if !record.path.as_os_str().is_empty() {
            let path_str = record.path.to_string_lossy().to_lowercase();
            path_str.ends_with(r"\softwaredistribution\download")
                || path_str.ends_with("/softwaredistribution/download")
        } else {
            name == "download"
                && _index
                    .get_record(record.parent_id)
                    .map(|p| p.name.eq_ignore_ascii_case("softwaredistribution"))
                    .unwrap_or(false)
        };

        if in_softwaredistribution {
            return Some((
                ArtifactCategory::WindowsUpdate,
                "Cached Windows Update downloaded installation packages".to_string(),
            ));
        }

        None
    }
}
