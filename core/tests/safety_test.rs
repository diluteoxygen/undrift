use git2::{Repository, Signature};
use std::fs::{create_dir_all, write};
use tempfile::TempDir;
use sweepie_core::model::candidate::GitRepoStatus;
use sweepie_core::model::{ArtifactCategory, ReclaimCandidate};
use sweepie_core::safety::SafetyPipeline;

#[test]
fn test_git_dirty_safety_check() {
    let temp_dir = TempDir::new().unwrap();
    let repo_dir = temp_dir.path();

    // 1. Initialize git repo
    let repo = Repository::init(repo_dir).unwrap();
    let sig = Signature::now("Tester", "test@example.com").unwrap();

    // Create Cargo.toml and .gitignore
    write(repo_dir.join("Cargo.toml"), "[package]\nname = \"foo\"").unwrap();
    write(repo_dir.join(".gitignore"), "target/\n").unwrap();

    // Add and commit files
    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new("Cargo.toml")).unwrap();
    index.add_path(std::path::Path::new(".gitignore")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
        .unwrap();

    // Create target directory (ignored)
    let target_dir = repo_dir.join("target");
    create_dir_all(&target_dir).unwrap();
    write(target_dir.join("build.log"), "done").unwrap();

    // Test when repo is clean
    let mut candidate = ReclaimCandidate::new(
        ArtifactCategory::RustTarget,
        target_dir.clone(),
        1024,
        None,
        1,
        true,
        "Rust target".to_string(),
        GitRepoStatus::NotInRepo,
    );

    SafetyPipeline::evaluate_candidate(&mut candidate);
    assert!(candidate.is_safe);
    assert_eq!(candidate.git_status, GitRepoStatus::Clean);
    assert!(candidate.default_selected);

    // Now introduce an uncommitted modification to Cargo.toml
    write(
        repo_dir.join("Cargo.toml"),
        "[package]\nname = \"foo-modified\"",
    )
    .unwrap();

    // Re-evaluate
    SafetyPipeline::evaluate_candidate(&mut candidate);
    assert!(!candidate.is_safe);
    assert!(!candidate.default_selected);
    match candidate.git_status {
        GitRepoStatus::Dirty { modified_count } => {
            assert!(modified_count >= 1);
        }
        _ => panic!("Expected GitRepoStatus::Dirty"),
    }
    assert!(candidate.safety_reason.contains("uncommitted change"));
}

#[test]
fn test_git_cached_monorepo_evaluation() {
    let temp_dir = TempDir::new().unwrap();
    let repo_dir = temp_dir.path();

    let repo = Repository::init(repo_dir).unwrap();
    let sig = Signature::now("Tester", "test@example.com").unwrap();

    write(repo_dir.join("root.txt"), "root").unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new("root.txt")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
        .unwrap();

    // Create 5 subproject candidate directories
    let mut candidates = Vec::new();
    for i in 0..5 {
        let pkg_dir = repo_dir.join(format!("packages/pkg_{i}/node_modules"));
        create_dir_all(&pkg_dir).unwrap();
        write(pkg_dir.join("dep.js"), "console.log(1);").unwrap();

        candidates.push(ReclaimCandidate::new(
            ArtifactCategory::NodeModules,
            pkg_dir,
            1024,
            None,
            1,
            true,
            "Node modules".to_string(),
            GitRepoStatus::NotInRepo,
        ));
    }

    // Dirtify the monorepo root
    write(repo_dir.join("root.txt"), "root modified").unwrap();

    // Evaluate in parallel with cached checker
    SafetyPipeline::evaluate_candidates(&mut candidates);

    assert_eq!(candidates.len(), 5);
    for c in &candidates {
        assert!(
            !c.is_safe,
            "All candidates in dirty repo must be marked unsafe"
        );
        assert!(!c.default_selected);
        match &c.git_status {
            GitRepoStatus::Dirty { modified_count } => {
                assert!(*modified_count >= 1);
            }
            _ => panic!("Expected GitRepoStatus::Dirty for all candidates"),
        }
    }
}
