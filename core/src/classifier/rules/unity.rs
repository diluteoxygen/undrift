use super::ClassificationRule;
use crate::model::category::ArtifactCategory;
use crate::model::file_record::FileRecord;
use crate::scanner::ScanIndex;

pub struct UnityRule;

impl ClassificationRule for UnityRule {
    fn name(&self) -> &'static str {
        "Unity Build Artifacts"
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
        if name == "library" || name == "temp" {
            let parent_id = record.parent_id;
            // Unity projects have an Assets folder and ProjectSettings folder
            if index.has_child_with_name(parent_id, "assets")
                || index.has_child_with_name(parent_id, "projectsettings")
            {
                return Some((
                    ArtifactCategory::Unity,
                    format!(
                        "Unity editor cache '{name}/' directory (re-generated on project open)"
                    ),
                ));
            }
        }

        None
    }
}
