pub mod cloud_storage;
pub mod git;
pub mod in_use;
pub mod reparse;

use crate::model::candidate::ReclaimCandidate;
use cloud_storage::CloudStorageChecker;
use git::GitSafetyChecker;
use in_use::InUseChecker;
use rayon::prelude::*;
use reparse::ReparseChecker;

pub struct SafetyPipeline;

impl SafetyPipeline {
    /// Validates safety guarantees for all candidates in parallel using rayon
    pub fn evaluate_candidates(candidates: &mut [ReclaimCandidate]) {
        let git_checker = GitSafetyChecker::new();
        candidates.par_iter_mut().for_each(|candidate| {
            Self::evaluate_candidate_with_git(candidate, &git_checker);
        });
    }

    pub fn evaluate_candidate(candidate: &mut ReclaimCandidate) {
        let git_checker = GitSafetyChecker::new();
        Self::evaluate_candidate_with_git(candidate, &git_checker);
    }

    pub fn evaluate_candidate_with_git(
        candidate: &mut ReclaimCandidate,
        git_checker: &GitSafetyChecker,
    ) {
        // 1. Check Git status (uses cached repo status)
        let (git_status, git_err) = git_checker.check_path(&candidate.path);
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

    /// Lightweight re-check to run immediately before deletion to prevent TOCTOU
    pub fn pre_clean_check(path: &std::path::Path) -> Option<String> {
        let path_buf = path.to_path_buf();

        if let Some(err_msg) = GitSafetyChecker::check_path_once(&path_buf).1 {
            return Some(err_msg);
        }

        if let Some(cloud_err) = CloudStorageChecker::is_cloud_placeholder(&path_buf) {
            return Some(cloud_err);
        }

        if let Some(lock_err) = InUseChecker::is_locked(&path_buf) {
            return Some(lock_err);
        }

        if let Some(reparse_err) = ReparseChecker::is_junction_or_symlink(&path_buf) {
            return Some(reparse_err);
        }

        None
    }
}
