use super::ClassificationRule;
use crate::model::category::ArtifactCategory;
use crate::model::file_record::FileRecord;
use crate::scanner::ScanIndex;

pub struct NodeRule;

impl ClassificationRule for NodeRule {
    fn name(&self) -> &'static str {
        "Node.js Dependencies"
    }

    fn evaluate(
        &self,
        index: &ScanIndex,
        record: &FileRecord,
    ) -> Option<(ArtifactCategory, String)> {
        if !record.is_dir {
            return None;
        }

        if record.name.eq_ignore_ascii_case("node_modules") {
            // Check if parent directory contains package.json
            if index.has_child_with_name(record.parent_id, "package.json") {
                return Some((
                    ArtifactCategory::NodeModules,
                    "Node.js dependencies next to package.json (re-installable via npm/yarn/pnpm)"
                        .to_string(),
                ));
            }
        }

        None
    }
}
