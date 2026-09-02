use super::ClassificationRule;
use crate::model::category::ArtifactCategory;
use crate::model::file_record::FileRecord;
use crate::scanner::ScanIndex;

pub struct IdeRule;

impl ClassificationRule for IdeRule {
    fn name(&self) -> &'static str {
        "IDE Workspace Caches"
    }

    fn evaluate(
        &self,
        index: &ScanIndex,
        record: &FileRecord,
    ) -> Option<(ArtifactCategory, String)> {
        if !record.is_dir {
            return None;
        }

        let name = record.name.to_lowercase();

        // Visual Studio .vs folder
        if name == ".vs" {
            let parent_id = record.parent_id;
            if index.has_child_with_extension(parent_id, "sln") {
                return Some((
                    ArtifactCategory::VisualStudio,
                    "Visual Studio solution index and IntelliSense cache (.vs)".to_string(),
                ));
            }
        }

        // JetBrains .idea folder
        if name == ".idea" {
            return Some((
                ArtifactCategory::JetBrains,
                "JetBrains IDE project settings and index cache (.idea)".to_string(),
            ));
        }

        None
    }
}
