use std::fs::{create_dir_all, write};
use std::time::Instant;
use tempfile::TempDir;
use undrift_core::classifier::ClassifierPipeline;
use undrift_core::safety::SafetyPipeline;
use undrift_core::scanner::VolumeScanner;
use undrift_core::scanner::dir_walk::DirWalkScanner;

fn main() {
    println!("\n═════════════════════════════════════════════════════════════════════════");
    println!("  UNDRIFT SCAN & CLASSIFICATION BENCHMARK HARNESS");
    println!("═════════════════════════════════════════════════════════════════════════");

    let temp_dir = TempDir::new().expect("Failed to create temp fixture directory");
    let root = temp_dir.path();
    println!(
        "  Generating synthetic developer machine fixture in: {}",
        root.display()
    );

    let fixture_start = Instant::now();
    let mut total_created_files = 0;

    // 1. Monorepo with 12 packages
    let monorepo = root.join("workspace/my-monorepo");
    create_dir_all(&monorepo).unwrap();
    write(
        monorepo.join("package.json"),
        r#"{"name":"monorepo","private":true}"#,
    )
    .unwrap();
    total_created_files += 1;

    for i in 0..12 {
        let pkg_dir = monorepo.join(format!("packages/app_{i}"));
        let nm_dir = pkg_dir.join("node_modules");
        create_dir_all(&nm_dir).unwrap();
        write(
            pkg_dir.join("package.json"),
            format!(r#"{{"name":"app_{i}"}}"#),
        )
        .unwrap();
        total_created_files += 1;

        for j in 0..15 {
            let dep_dir = nm_dir.join(format!("dep_{j}"));
            create_dir_all(&dep_dir).unwrap();
            write(dep_dir.join("index.js"), "module.exports = {};").unwrap();
            write(
                dep_dir.join("package.json"),
                format!(r#"{{"name":"dep_{j}"}}"#),
            )
            .unwrap();
            total_created_files += 2;
        }
    }

    // 2. 6 Rust projects with Cargo.toml + target/debug
    for i in 0..6 {
        let rust_proj = root.join(format!("workspace/rust_crate_{i}"));
        let target_dir = rust_proj.join("target/debug/deps");
        create_dir_all(&target_dir).unwrap();
        write(
            rust_proj.join("Cargo.toml"),
            format!("[package]\nname = \"crate_{i}\""),
        )
        .unwrap();
        total_created_files += 1;

        for j in 0..20 {
            write(
                target_dir.join(format!("lib_{j}.rlib")),
                vec![0u8; 1024 * 50],
            )
            .unwrap();
            total_created_files += 1;
        }
    }

    // 3. 4 Python projects with .venv
    for i in 0..4 {
        let py_proj = root.join(format!("workspace/py_project_{i}"));
        let venv_dir = py_proj.join(".venv/lib/site-packages");
        create_dir_all(&venv_dir).unwrap();
        write(py_proj.join("pyproject.toml"), "[project]\nname = \"proj\"").unwrap();
        total_created_files += 1;

        for j in 0..10 {
            write(venv_dir.join(format!("pkg_{j}.py")), "def foo(): pass").unwrap();
            total_created_files += 1;
        }
    }

    // 4. Downloads folder with installer files
    let downloads = root.join("Users/Developer/Downloads");
    create_dir_all(&downloads).unwrap();
    for i in 0..5 {
        write(
            downloads.join(format!("installer_{i}.exe")),
            vec![0u8; 1024 * 1024],
        )
        .unwrap();
        total_created_files += 1;
    }

    let fixture_duration = fixture_start.elapsed();
    println!(
        "  Created {} synthetic files in {:.2}ms",
        total_created_files,
        fixture_duration.as_secs_f64() * 1000.0
    );
    println!("─────────────────────────────────────────────────────────────────────────");

    // Benchmark Phase 1: Ingestion
    let scan_start = Instant::now();
    let scanner = DirWalkScanner::new();
    let index = scanner.scan(root).expect("Scanner failed");
    let scan_duration = scan_start.elapsed();

    let records_count = index.records.len();
    let records_per_sec = records_count as f64 / scan_duration.as_secs_f64();
    println!(
        "  1. Ingestion / Scanner Phase : {:.2}ms ({} records, {:.0} records/sec)",
        scan_duration.as_secs_f64() * 1000.0,
        records_count,
        records_per_sec
    );

    // Benchmark Phase 2: Classification
    let classify_start = Instant::now();
    let pipeline = ClassifierPipeline::new(0);
    let mut candidates = pipeline.classify(&index);
    let classify_duration = classify_start.elapsed();
    println!(
        "  2. Classification Phase     : {:.2}ms ({} candidates surfaced)",
        classify_duration.as_secs_f64() * 1000.0,
        candidates.len()
    );

    // Benchmark Phase 3: Parallel Safety Evaluation
    let safety_start = Instant::now();
    SafetyPipeline::evaluate_candidates(&mut candidates);
    let safety_duration = safety_start.elapsed();
    println!(
        "  3. Parallel Safety Pipeline : {:.2}ms (cached git & rayon parallel)",
        safety_duration.as_secs_f64() * 1000.0
    );

    let total_pipeline = scan_duration + classify_duration + safety_duration;
    println!("─────────────────────────────────────────────────────────────────────────");
    println!(
        "  TOTAL PIPELINE LATENCY      : {:.2}ms",
        total_pipeline.as_secs_f64() * 1000.0
    );
    println!("═════════════════════════════════════════════════════════════════════════\n");
}
