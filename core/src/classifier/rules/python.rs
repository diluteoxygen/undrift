use super::ClassificationRule;
use crate::model::category::ArtifactCategory;
use crate::model::file_record::FileRecord;
use crate::scanner::ScanIndex;

pub struct PythonRule;

impl ClassificationRule for PythonRule {
    fn name(&self) -> &'static str {
        "Python Artifacts"
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

        // Check __pycache__
        if name == "__pycache__" {
            return Some((
                ArtifactCategory::PythonCache,
                "Python bytecode cache folder (regenerated automatically on execution)".to_string(),
            ));
        }

        // Check virtual environments
        if name == ".venv" || name == "venv" || name == "env" || name == ".env_py" {
            let parent_id = record.parent_id;
            if index.has_child_with_name(parent_id, "pyproject.toml")
                || index.has_child_with_name(parent_id, "requirements.txt")
                || index.has_child_with_name(parent_id, "Pipfile")
                || index.has_child_with_name(parent_id, "setup.py")
            {
                return Some((
                    ArtifactCategory::PythonVenv,
                    format!("Python virtual environment '{name}' next to project manifest"),
                ));
            }
        }

        None
    }
}
