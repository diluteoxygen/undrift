pub mod rules;

use crate::model::candidate::GitRepoStatus;
use crate::model::candidate::ReclaimCandidate;
use crate::scanner::ScanIndex;
use rules::ClassificationRule;
use rules::dotnet_java::DotnetJavaRule;
use rules::downloads::DownloadsRule;
use rules::ide::IdeRule;
use rules::node::NodeRule;
use rules::python::PythonRule;
use rules::rust::RustRule;
use rules::unity::UnityRule;
use rules::windows_update::WindowsUpdateRule;
use std::collections::HashSet;
use std::path::PathBuf;

pub struct ClassifierPipeline {
    rules: Vec<Box<dyn ClassificationRule>>,
    min_size_bytes: u64,
}

impl Default for ClassifierPipeline {
    fn default() -> Self {
        Self::new(1024 * 1024) // 1 MB minimum by default
    }
}

impl ClassifierPipeline {
    pub fn new(min_size_bytes: u64) -> Self {
        let rules: Vec<Box<dyn ClassificationRule>> = vec![
            Box::new(NodeRule),
            Box::new(RustRule),
            Box::new(PythonRule),
            Box::new(DotnetJavaRule),
            Box::new(IdeRule),
            Box::new(UnityRule),
            Box::new(DownloadsRule::default()),
            Box::new(WindowsUpdateRule),
        ];

        Self {
            rules,
            min_size_bytes,
        }
    }

    pub fn with_stale_days(mut self, stale_days: i64) -> Self {
        // Update downloads rule
        self.rules
            .retain(|r| r.name() != "Stale Downloaded Installers");
        self.rules.push(Box::new(DownloadsRule::new(stale_days)));
        self
    }

    pub fn set_min_size(&mut self, min_size_bytes: u64) {
        self.min_size_bytes = min_size_bytes;
    }

    /// Evaluates all records in the index and returns discovered candidates
    pub fn classify(&self, index: &ScanIndex) -> Vec<ReclaimCandidate> {
        let mut candidates = Vec::new();
        let mut seen_paths: HashSet<PathBuf> = HashSet::new();

        // Sort records by depth / path length so parent candidates are discovered first
        let mut sorted_records: Vec<_> = index.records.values().collect();
        sorted_records.sort_by_key(|r| r.path.components().count());

        for record in sorted_records {
            // If an ancestor of this path has already been matched as a candidate, skip child
            let has_ancestor = seen_paths
                .iter()
                .any(|parent| record.path.starts_with(parent));
            if has_ancestor {
                continue;
            }

            for rule in &self.rules {
                if let Some((category, reason)) = rule.evaluate(index, record) {
                    let (size_bytes, file_count, latest_modified) = if record.is_dir {
                        index.calculate_subtree_stats(record.id)
                    } else {
                        (record.size_bytes, 1, record.modified_at)
                    };

                    if size_bytes < self.min_size_bytes {
                        continue;
                    }

                    let modified = latest_modified.or(record.modified_at);

                    let candidate = ReclaimCandidate::new(
                        category,
                        record.path.clone(),
                        size_bytes,
                        modified,
                        file_count,
                        true, // initially true, safety pipeline will validate
                        reason,
                        GitRepoStatus::NotInRepo,
                    );

                    seen_paths.insert(record.path.clone());
                    candidates.push(candidate);
                    break;
                }
            }
        }

        candidates
    }
}
