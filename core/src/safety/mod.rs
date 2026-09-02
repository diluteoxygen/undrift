pub mod cloud_storage;
pub mod git;
pub mod in_use;
pub mod reparse;

use crate::model::candidate::ReclaimCandidate;
use cloud_storage::CloudStorageChecker;
use git::GitSafetyChecker;
use in_use::InUseChecker;
use reparse::ReparseChecker;

pub struct SafetyPipeline;

impl SafetyPipeline {
    /// Validates safety guarantees for all candidates
    pub fn evaluate_candidates(candidates: &mut [ReclaimCandidate]) {
        for candidate in candidates.iter_mut() {
            Self::evaluate_candidate(candidate);
        }
    }

    pub fn evaluate_candidate(candidate: &mut ReclaimCandidate) {
        // 1. Check Git status
        let (git_status, git_err) = GitSafetyChecker::check_path(&candidate.path);
        candidate.git_status = git_status;

        if let Some(err_msg) = git_err {
            candidate.is_safe = false;
            candidate.safety_reason = err_msg;
            candidate.default_selected = false;
            return;
        }

        // 2. Check cloud storage / OneDrive reparse placeholders
        if let Some(cloud_err) = CloudStorageChecker::is_cloud_placeholder(&candidate.path) {
            candidate.is_safe = false;
            candidate.safety_reason = cloud_err;
            candidate.default_selected = false;
            return;
        }

        // 3. Check for open file handles / locks
        if let Some(lock_err) = InUseChecker::is_locked(&candidate.path) {
            candidate.is_safe = false;
            candidate.safety_reason = lock_err;
            candidate.default_selected = false;
            return;
        }

        // 4. Check junctions and symlinks
        if let Some(reparse_err) = ReparseChecker::is_junction_or_symlink(&candidate.path) {
            candidate.is_safe = false;
            candidate.safety_reason = reparse_err;
            candidate.default_selected = false;
            return;
        }

        // All checks passed
        candidate.is_safe = true;
        candidate.default_selected = true;
    }
}
