pub mod dotnet_java;
pub mod downloads;
pub mod ide;
pub mod node;
pub mod python;
pub mod rust;
pub mod unity;
pub mod windows_update;

use crate::model::category::ArtifactCategory;
use crate::model::file_record::FileRecord;
use crate::scanner::ScanIndex;

pub trait ClassificationRule: Send + Sync {
    fn name(&self) -> &'static str;

    /// Evaluates if a given record matches this rule.
    /// Returns Some((category, one_line_reason)) if matched.
    fn evaluate(
        &self,
        index: &ScanIndex,
        record: &FileRecord,
    ) -> Option<(ArtifactCategory, String)>;
}
