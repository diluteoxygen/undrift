use std::fs::{create_dir_all, write};
use tempfile::TempDir;
use undrift_core::classifier::ClassifierPipeline;
use undrift_core::model::ArtifactCategory;
use undrift_core::scanner::VolumeScanner;
use undrift_core::scanner::dir_walk::DirWalkScanner;

#[test]
fn test_classify_node_modules_and_rust_target() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Setup Node.js project: package.json + node_modules
    let node_dir = root.join("my-node-app");
    create_dir_all(node_dir.join("node_modules").join("lodash")).unwrap();
    write(node_dir.join("package.json"), r#"{"name": "test"}"#).unwrap();
    write(
        node_dir
            .join("node_modules")
            .join("lodash")
            .join("index.js"),
        "module.exports = {};",
    )
    .unwrap();

    // Setup Rust project: Cargo.toml + target
    let rust_dir = root.join("my-rust-app");
    create_dir_all(rust_dir.join("target").join("debug")).unwrap();
    write(
        rust_dir.join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"",
    )
    .unwrap();
    write(
        rust_dir.join("target").join("debug").join("app.exe"),
        vec![0u8; 1024 * 100],
    )
    .unwrap();

    // Setup Python project: pyproject.toml + .venv
    let py_dir = root.join("my-py-app");
    create_dir_all(py_dir.join(".venv").join("bin")).unwrap();
    write(py_dir.join("pyproject.toml"), "[project]\nname = \"test\"").unwrap();
    write(py_dir.join(".venv").join("bin").join("python"), "dummy").unwrap();

    let scanner = DirWalkScanner::new();
    let index = scanner.scan(root).unwrap();

    let pipeline = ClassifierPipeline::new(0); // 0 minimum size for tests
    let candidates = pipeline.classify(&index);

    let categories: Vec<ArtifactCategory> = candidates.iter().map(|c| c.category).collect();
    assert!(categories.contains(&ArtifactCategory::NodeModules));
    assert!(categories.contains(&ArtifactCategory::RustTarget));
    assert!(categories.contains(&ArtifactCategory::PythonVenv));

    let rust_candidate = candidates
        .iter()
        .find(|c| c.category == ArtifactCategory::RustTarget)
        .unwrap();
    assert!(rust_candidate.size_bytes >= 100 * 1024);
    assert_eq!(rust_candidate.path, rust_dir.join("target"));
}
