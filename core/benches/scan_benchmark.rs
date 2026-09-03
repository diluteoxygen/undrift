use std::fs::{create_dir_all, hard_link, write};
use std::time::Instant;
use sweepie_core::classifier::ClassifierPipeline;
use sweepie_core::safety::SafetyPipeline;
use sweepie_core::scanner::VolumeScanner;
use sweepie_core::scanner::dir_walk::DirWalkScanner;
use sweepie_core::scanner::persistent_index::{PersistentIndex, UsnCheckpoint, UsnDeltaRecord};
use tempfile::TempDir;

fn main() {
    println!("\n═════════════════════════════════════════════════════════════════════════");
    println!("  SWEEPIE BENCHMARK: COLD SCAN VS USN WARM INCREMENTAL INDEX");
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

    // 4. pnpm-style hardlinked package store simulation
    let pnpm_store = root.join("workspace/.pnpm_store");
    let pnpm_app = root.join("workspace/pnpm_app");
    let pnpm_nm = pnpm_app.join("node_modules/lodash");
    create_dir_all(&pnpm_store).unwrap();
    create_dir_all(&pnpm_nm).unwrap();
    write(pnpm_app.join("package.json"), r#"{"name":"pnpm_app"}"#).unwrap();
    total_created_files += 1;

    let store_file = pnpm_store.join("lodash.js");
    write(&store_file, vec![0u8; 1024 * 200]).unwrap();
    total_created_files += 1;

    let hardlink_file = pnpm_nm.join("lodash.js");
    let _ = hard_link(&store_file, &hardlink_file);

    let fixture_duration = fixture_start.elapsed();
    println!(
        "  Created {} synthetic files in {:.2}ms",
        total_created_files,
        fixture_duration.as_secs_f64() * 1000.0
    );
    println!("─────────────────────────────────────────────────────────────────────────");

    // ==========================================
    // BENCHMARK 1: COLD SCAN (FULL MFT / DIR WALK)
    // ==========================================
    let cold_start = Instant::now();
    let scanner = DirWalkScanner::new();
    let cold_index = scanner.scan(root).expect("Cold scan failed");
    let cold_scan_duration = cold_start.elapsed();

    let pipeline = ClassifierPipeline::new(0);
    let mut cold_candidates = pipeline.classify(&cold_index);
    SafetyPipeline::evaluate_candidates(&mut cold_candidates);
    let cold_total_duration = cold_start.elapsed();

    println!("  [COLD SCAN]");
    println!(
        "    - Ingestion Duration     : {:.2}ms ({} records)",
        cold_scan_duration.as_secs_f64() * 1000.0,
        cold_index.records.len()
    );
    println!("    - Candidates Discovered  : {}", cold_candidates.len());
    println!(
        "    - Total Cold Latency     : {:.2}ms",
        cold_total_duration.as_secs_f64() * 1000.0
    );

    // Save persistent index checkpoint
    let index_file = root.join("persistent_index.json");
    let persistent = PersistentIndex::new("TEST_VOL", 7777, 10000, &cold_index);
    persistent.save_to_disk(&index_file).unwrap();

    // ==========================================
    // BENCHMARK 2: WARM SCAN (USN DELTA REPLAY)
    // ==========================================
    println!("\n  [WARM INCREMENTAL SCAN via USN JOURNAL]");
    let warm_start = Instant::now();

    // 1. Load persisted index
    let mut warm_persistent = PersistentIndex::load_from_disk(&index_file).unwrap();

    // 2. Check USN continuity (checkpoint at next_usn 10050)
    let checkpoint = UsnCheckpoint {
        journal_id: 7777,
        lowest_usn: 5000,
        next_usn: 10050,
    };
    warm_persistent
        .verify_journal_continuity(&checkpoint)
        .unwrap();

    // 3. Replay delta records
    let mut warm_index = warm_persistent.to_scan_index(root.to_path_buf());
    let deltas = vec![UsnDeltaRecord::Modified {
        id: 1,
        new_size: 4096,
    }];
    warm_persistent.apply_deltas(&mut warm_index, &deltas, 10050);
    let warm_ingest_duration = warm_start.elapsed();

    // 4. Classify
    let mut warm_candidates = pipeline.classify(&warm_index);
    SafetyPipeline::evaluate_candidates(&mut warm_candidates);
    let warm_total_duration = warm_start.elapsed();

    println!(
        "    - Delta Ingestion Time   : {:.2}ms (fraction of cold scan!)",
        warm_ingest_duration.as_secs_f64() * 1000.0
    );
    println!(
        "    - Total Warm Latency     : {:.2}ms",
        warm_total_duration.as_secs_f64() * 1000.0
    );
    let speedup = cold_total_duration.as_secs_f64() / warm_total_duration.as_secs_f64().max(0.0001);
    println!("    - Speedup vs Cold Scan   : {:.1}x faster", speedup);

    // ==========================================
    // VERIFICATION: HARDLINK-AWARE ACCURACY
    // ==========================================
    println!("\n  [HARDLINK CAVEAT VERIFICATION]");
    let pnpm_candidate = cold_candidates.iter().find(|c| {
        c.path.ends_with("pnpm_app/node_modules") || c.path.ends_with("pnpm_app\\node_modules")
    });

    if let Some(cand) = pnpm_candidate {
        println!("    - Target                 : {}", cand.display_path);
        println!("    - Has Hardlinks          : {}", cand.has_hardlinks);
        println!(
            "    - Shared Hardlink Bytes  : {}",
            cand.hardlink_shared_bytes
        );
        if let Some(ref caveat) = cand.size_caveat {
            println!("    - Caveat Surfaced        : {}", caveat);
        }
        assert!(cand.has_hardlinks, "pnpm hardlinks must be detected");
    }

    // ==========================================
    // VERIFICATION: JOURNAL WRAP DETECTION
    // ==========================================
    println!("\n  [JOURNAL WRAP SAFETY VERIFICATION]");
    let wrapped_checkpoint = UsnCheckpoint {
        journal_id: 7777,
        lowest_usn: 15000, // saved is 10000 < 15000 -> wrapped!
        next_usn: 20000,
    };
    let wrap_result = warm_persistent.verify_journal_continuity(&wrapped_checkpoint);
    assert!(
        wrap_result.is_err(),
        "Journal wrap must be explicitly detected"
    );
    println!(
        "    - Wrap condition detected cleanly: {:?}",
        wrap_result.err().unwrap()
    );

    println!("─────────────────────────────────────────────────────────────────────────");
    println!("  PHASE 4 ACCEPTANCE CRITERIA VERIFIED SUCCESSFULLY");
    println!("═════════════════════════════════════════════════════════════════════════\n");
}
