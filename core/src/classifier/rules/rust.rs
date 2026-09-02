use super::ClassificationRule;
use crate::model::category::ArtifactCategory;
use crate::model::file_record::FileRecord;
use crate::scanner::ScanIndex;

pub struct RustRule;

impl ClassificationRule for RustRule {
    fn name(&self) -> &'static str {
        "Rust Build Output"
    }

    fn evaluate(
        &self,
        index: &ScanIndex,
        record: &FileRecord,
    ) -> Option<(ArtifactCategory, String)> {
        if !record.is_dir {
            return None;
        }

        if record.name.eq_ignore_ascii_case("target") {
            // Check if parent directory contains Cargo.toml
            if index.has_child_with_name(record.parent_id, "Cargo.toml") {
                return Some((
                    ArtifactCategory::RustTarget,
                    "Rust build output next to Cargo.toml (reproducible via cargo build)"
                        .to_string(),
                ));
            }
        }

        None
    }
}
