use std::fs::{create_dir_all, write};
use tempfile::TempDir;
use undrift_core::cleaner::CleanExecutor;

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
