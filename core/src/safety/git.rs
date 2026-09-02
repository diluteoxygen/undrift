use crate::model::candidate::GitRepoStatus;
use git2::{Repository, StatusOptions};
use std::path::Path;

pub struct GitSafetyChecker;

impl GitSafetyChecker {
    /// Checks if a path is inside a Git repository, and whether that repo has uncommitted changes.
    pub fn check_path(target_path: &Path) -> (GitRepoStatus, Option<String>) {
        // Attempt to discover Git repository from path
        let repo = match Repository::discover(target_path) {
            Ok(r) => r,
            Err(_) => return (GitRepoStatus::NotInRepo, None),
        };

        let repo_workdir = match repo.workdir() {
            Some(w) => w.to_path_buf(),
            None => return (GitRepoStatus::NotInRepo, None),
        };

        // Prepare status options: check untracked and modified files, ignoring gitignored files
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        opts.include_ignored(false);
        opts.recurse_untracked_dirs(false);

        let statuses = match repo.statuses(Some(&mut opts)) {
            Ok(s) => s,
            Err(e) => {
                return (
                    GitRepoStatus::Error {
                        message: e.to_string(),
                    },
                    Some(format!("Git error reading repository status: {e}")),
                );
            }
        };

        let mut uncommitted_count = 0;
        for entry in statuses.iter() {
            let status = entry.status();
            if status.is_wt_modified()
                || status.is_wt_new()
                || status.is_wt_deleted()
                || status.is_wt_renamed()
                || status.is_index_modified()
                || status.is_index_new()
                || status.is_index_deleted()
                || status.is_index_renamed()
            {
                // Ensure the dirty file is not inside the candidate path itself (which is ignored build output)
                if let Ok(entry_path) = entry.path() {
                    let full_entry_path = repo_workdir.join(entry_path);
                    if !full_entry_path.starts_with(target_path) {
                        uncommitted_count += 1;
                    }
                } else {
                    uncommitted_count += 1;
                }
            }
        }

        if uncommitted_count > 0 {
            let reason = format!(
                "Git repository at '{}' has {} uncommitted change(s). Commit or stash before cleaning.",
                repo_workdir.display(),
                uncommitted_count
            );
            (
                GitRepoStatus::Dirty {
                    modified_count: uncommitted_count,
                },
                Some(reason),
            )
        } else {
            (GitRepoStatus::Clean, None)
        }
    }
}
