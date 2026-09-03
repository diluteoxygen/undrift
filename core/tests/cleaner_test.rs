use std::fs::{create_dir_all, write};
use sweepie_core::cleaner::CleanExecutor;
use tempfile::TempDir;

#[test]
fn test_cleaner_dry_run_and_permanent() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let target_dir = root.join("artifact_dir");
    create_dir_all(&target_dir).unwrap();
    let test_file = target_dir.join("temp.bin");
    write(&test_file, vec![1u8; 2048]).unwrap();

    let targets = vec![(target_dir.clone(), 2048)];

    // Dry run
    let report_dry = CleanExecutor::clean_targets(&targets, false, true);
    assert_eq!(report_dry.total_reclaimed_bytes, 2048);
    assert!(report_dry.is_dry_run);
    assert!(target_dir.exists(), "Dry run must not delete files");

    // Permanent deletion
    let report_perm = CleanExecutor::clean_targets(&targets, true, false);
    assert_eq!(report_perm.total_reclaimed_bytes, 2048);
    assert!(report_perm.was_permanent);
    assert!(
        !target_dir.exists(),
        "Permanent delete must remove directory"
    );
}

#[test]
fn test_cleaner_skips_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    let fake_path = root.join("does_not_exist");
    let targets = vec![(fake_path.clone(), 1000)];

    let report = CleanExecutor::clean_targets(&targets, true, false);
    assert_eq!(report.total_reclaimed_bytes, 0);
    assert_eq!(report.succeeded.len(), 0);
    assert_eq!(report.failed.len(), 1);
    assert_eq!(report.failed[0].path, fake_path);
    assert_eq!(report.failed[0].error_message, "Path does not exist");
}

#[test]
fn test_cleaner_pre_clean_check_git_dirty() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Simulate git repo setup
    let git_dir = root.join(".git");
    create_dir_all(&git_dir).unwrap();
    write(root.join(".git").join("HEAD"), "ref: refs/heads/main").unwrap();

    // Create a modified file outside the target path
    let dirty_file = root.join("modified.txt");
    write(&dirty_file, "content").unwrap();

    // Create the target path
    let target_dir = root.join("target");
    create_dir_all(&target_dir).unwrap();
    write(target_dir.join("build.o"), "binary").unwrap();

    std::process::Command::new("git")
        .current_dir(root)
        .arg("init")
        .output()
        .unwrap();

    std::process::Command::new("git")
        .current_dir(root)
        .arg("add")
        .arg("modified.txt")
        .output()
        .unwrap();

    let targets = vec![(target_dir.clone(), 100)];
    let report = CleanExecutor::clean_targets(&targets, true, false);

    // Because the repo is dirty, safety re-check should fail
    assert_eq!(report.succeeded.len(), 0);
    assert_eq!(report.failed.len(), 1);
    assert!(
        report.failed[0]
            .error_message
            .contains("Safety re-check failed")
    );
}
