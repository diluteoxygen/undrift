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
        let mut candidate_ids: HashSet<u64> = HashSet::new();

        // Sort records by depth from root so parent candidates are discovered first
        let mut sorted_records: Vec<_> = index.records.values().collect();
        sorted_records.sort_by_key(|r| {
            if !r.path.as_os_str().is_empty() {
                r.path.components().count()
            } else {
                let mut depth = 0usize;
                let mut curr = r.parent_id;
                while curr != 0 && curr != 5 && depth < 64 {
                    depth += 1;
                    match index.get_record(curr) {
                        Some(parent) => {
                            if parent.parent_id == curr {
                                break;
                            }
                            curr = parent.parent_id;
                        }
                        None => break,
                    }
                }
                depth
            }
        });

        for record in sorted_records {
            // Ancestor check: climb parent chain to verify no ancestor is already an accepted candidate
            let mut is_dominated = false;
            let mut curr = record.parent_id;
            while curr != 0 && curr != 5 {
                if candidate_ids.contains(&curr) {
                    is_dominated = true;
                    break;
                }
                match index.get_record(curr) {
                    Some(parent) => {
                        if parent.parent_id == curr {
                            break;
                        }
                        curr = parent.parent_id;
                    }
                    None => break,
                }
            }
            if is_dominated {
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
                    let target_path = index.resolve_path(record.id);

                    let candidate = ReclaimCandidate::new(
                        category,
                        target_path,
                        size_bytes,
                        modified,
                        file_count,
                        true, // initially true, safety pipeline will validate
                        reason,
                        GitRepoStatus::NotInRepo,
                    );

                    candidate_ids.insert(record.id);
                    candidates.push(candidate);
                    break;
                }
            }
        }

        candidates
    }
}
