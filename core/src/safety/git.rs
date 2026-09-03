use crate::model::candidate::GitRepoStatus;
use git2::{Repository, StatusOptions};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct CachedRepoStatus {
    pub repo_workdir: PathBuf,
    pub dirty_files: Vec<PathBuf>,
}

#[derive(Default)]
pub struct GitSafetyChecker {
    // Maps repository workdir path to its dirty files or git error
    repo_cache: RwLock<HashMap<PathBuf, Result<Arc<CachedRepoStatus>, String>>>,
    // Paths verified not to be inside any git repository
    not_in_repo: RwLock<Vec<PathBuf>>,
}

impl GitSafetyChecker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fast, cached check: checks if target_path is inside a Git repository,
    /// reusing status results if the enclosing repository was already scanned.
    pub fn check_path(&self, target_path: &Path) -> (GitRepoStatus, Option<String>) {
        // 1. Check if an ancestor or this path is already known to not be in a Git repo
        {
            let non_repos = self.not_in_repo.read().unwrap();
            if non_repos.iter().any(|p| target_path.starts_with(p)) {
                return (GitRepoStatus::NotInRepo, None);
            }
        }

        // 2. Check if this target belongs to an already cached repo workdir
        {
            let cache = self.repo_cache.read().unwrap();
            for (workdir, result) in cache.iter() {
                if target_path.starts_with(workdir) {
                    return match result {
                        Ok(status) => {
                            Self::evaluate_candidate_against_dirty_files(target_path, status)
                        }
                        Err(err_msg) => (
                            GitRepoStatus::Error {
                                message: err_msg.clone(),
                            },
                            Some(err_msg.clone()),
                        ),
                    };
                }
            }
        }

        // 3. Not in cache yet: discover repository
        let repo = match Repository::discover(target_path) {
            Ok(r) => r,
            Err(_) => {
                let mut non_repos = self.not_in_repo.write().unwrap();
                non_repos.push(target_path.to_path_buf());
                return (GitRepoStatus::NotInRepo, None);
            }
        };

        let repo_workdir = match repo.workdir() {
            Some(w) => w.to_path_buf(),
            None => {
                let mut non_repos = self.not_in_repo.write().unwrap();
                non_repos.push(target_path.to_path_buf());
                return (GitRepoStatus::NotInRepo, None);
            }
        };

        // Check again under write lock in case another thread populated it in the interim
        let mut cache = self.repo_cache.write().unwrap();
        if let Some(cached) = cache.get(&repo_workdir) {
            return match cached {
                Ok(status) => Self::evaluate_candidate_against_dirty_files(target_path, status),
                Err(err_msg) => (
                    GitRepoStatus::Error {
                        message: err_msg.clone(),
                    },
                    Some(err_msg.clone()),
                ),
            };
        }

        // Perform single status scan for this repo
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        opts.include_ignored(false);
        opts.recurse_untracked_dirs(false);

        let status_result = match repo.statuses(Some(&mut opts)) {
            Ok(statuses) => {
                let mut dirty_files = Vec::new();
                for entry in statuses.iter() {
                    let status = entry.status();
                    let is_dirty = status.is_wt_modified()
                        || status.is_wt_new()
                        || status.is_wt_deleted()
                        || status.is_wt_renamed()
                        || status.is_index_modified()
                        || status.is_index_new()
                        || status.is_index_deleted()
                        || status.is_index_renamed();

                    if is_dirty && let Ok(entry_path) = entry.path() {
                        dirty_files.push(PathBuf::from(entry_path));
                    }
                }
                Ok(Arc::new(CachedRepoStatus {
                    repo_workdir: repo_workdir.clone(),
                    dirty_files,
                }))
            }
            Err(e) => Err(format!("Git error reading repository status: {e}")),
        };

        let eval_result = match &status_result {
            Ok(status) => Self::evaluate_candidate_against_dirty_files(target_path, status),
            Err(err_msg) => (
                GitRepoStatus::Error {
                    message: err_msg.clone(),
                },
                Some(err_msg.clone()),
            ),
        };

        cache.insert(repo_workdir, status_result);
        eval_result
    }

    fn evaluate_candidate_against_dirty_files(
        target_path: &Path,
        status: &CachedRepoStatus,
    ) -> (GitRepoStatus, Option<String>) {
        let mut uncommitted_count = 0;
        for rel_path in &status.dirty_files {
            let full_path = status.repo_workdir.join(rel_path);
            if !full_path.starts_with(target_path) {
                uncommitted_count += 1;
            }
        }

        if uncommitted_count > 0 {
            let reason = format!(
                "Git repository at '{}' has {} uncommitted change(s). Commit or stash before cleaning.",
                status.repo_workdir.display(),
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

    /// One-off uncached check for standalone callers
    pub fn check_path_once(target_path: &Path) -> (GitRepoStatus, Option<String>) {
        Self::new().check_path(target_path)
    }
}
