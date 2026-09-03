use clap::{Parser, Subcommand};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;
use undrift_core::classifier::ClassifierPipeline;
use undrift_core::cleaner::CleanExecutor;
use undrift_core::cleaner::history::HistoryManager;
use undrift_core::output::{ScanResultJson, ScanStreamEvent, print_human_table};
use undrift_core::safety::SafetyPipeline;
use undrift_core::scanner::VolumeScanner;
use undrift_core::scanner::dir_walk::DirWalkScanner;

#[derive(Parser)]
#[command(
    name = "undrift",
    author = "Vikrant Singh",
    version = "0.1.0",
    about = "Windows-native disk space reclaiming tool for developers. Fast MFT scanning, dev-aware judgment, zero telemetry."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan a drive or directory for reclaimable build artifacts and caches
    Scan {
        /// Drive or path to scan (e.g., C: or ./)
        #[arg(default_value = ".")]
        target: String,

        /// Output results as JSON for UI / machine consumption
        #[arg(long)]
        json: bool,

        /// Include skipped/unsafe candidates in output
        #[arg(long)]
        all: bool,

        /// Minimum candidate size in Megabytes to report (default: 1 MB)
        #[arg(long, default_value_t = 1)]
        min_size: u64,

        /// Stale installer threshold in days (default: 30 days)
        #[arg(long, default_value_t = 30)]
        stale_days: i64,

        /// Force NTFS MFT scan (Windows only)
        #[arg(long)]
        mft: bool,
    },

    /// Clean specified artifact paths
    Clean {
        /// Specific paths to clean
        #[arg(required = true)]
        paths: Vec<PathBuf>,

        /// Permanently delete instead of moving to Recycle Bin (Recycle Bin is default)
        #[arg(long)]
        permanent: bool,

        /// Dry-run simulation without removing files
        #[arg(long)]
        dry_run: bool,

        /// Skip interactive confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,

        /// Output results as JSON for UI / machine consumption
        #[arg(long)]
        json: bool,
    },

    /// View history log of previous cleanups and reclaimed space
    History,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Scan {
            target,
            json,
            all,
            min_size,
            stale_days,
            mft,
        }) => {
            run_scan(&target, json, all, min_size, stale_days, mft)?;
        }
        Some(Commands::Clean {
            paths,
            permanent,
            dry_run,
            yes,
            json,
        }) => {
            run_clean(&paths, permanent, dry_run, yes, json)?;
        }
        Some(Commands::History) => {
            run_history();
        }
        None => {
            // Default to scanning current directory
            run_scan(".", false, false, 1, 30, false)?;
        }
    }

    Ok(())
}

fn run_scan(
    target_str: &str,
    as_json: bool,
    show_all: bool,
    min_size_mb: u64,
    stale_days: i64,
    force_mft: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let target_path = Path::new(target_str);
    let start_total = Instant::now();

    #[allow(unused_variables, unused_mut)]
    let index_file = undrift_core::scanner::persistent_index::get_index_path_for_volume(target_str);
    #[allow(unused_variables, unused_mut)]
    let mut index_opt = None;

    #[cfg(windows)]
    {
        if let Ok(checkpoint) =
            undrift_core::scanner::persistent_index::query_usn_checkpoint(target_path)
            && let Ok(persistent) =
                undrift_core::scanner::persistent_index::PersistentIndex::load_from_disk(
                    &index_file,
                )
        {
            match persistent.verify_journal_continuity(&checkpoint) {
                Ok(()) => {
                    let mut warm_index = persistent.to_scan_index(target_path.to_path_buf());
                    warm_index.scan_duration = start_total.elapsed();
                    index_opt = Some(warm_index);
                }
                Err(e) => {
                    if !as_json {
                        eprintln!("  [USN Journal] {e}. Falling back to full MFT scan.");
                    }
                }
            }
        }
    }

    let index = match index_opt {
        Some(idx) => idx,
        None => {
            let scanner: Box<dyn VolumeScanner> = {
                #[cfg(windows)]
                {
                    let is_root_drive =
                        (target_str.len() <= 3 && target_str.contains(':')) || force_mft;
                    if is_root_drive {
                        Box::new(undrift_core::scanner::ntfs_mft::NtfsMftScanner::new())
                    } else {
                        Box::new(DirWalkScanner::new())
                    }
                }
                #[cfg(not(windows))]
                {
                    let _ = force_mft;
                    Box::new(DirWalkScanner::new())
                }
            };

            let scanned = scanner.scan(target_path)?;

            #[cfg(windows)]
            {
                if let Ok(checkpoint) =
                    undrift_core::scanner::persistent_index::query_usn_checkpoint(target_path)
                {
                    let persistent = undrift_core::scanner::persistent_index::PersistentIndex::new(
                        target_str,
                        checkpoint.journal_id,
                        checkpoint.next_usn,
                        &scanned,
                    );
                    let _ = persistent.save_to_disk(&index_file);
                }
            }

            scanned
        }
    };

    let mut pipeline = ClassifierPipeline::new(min_size_mb * 1024 * 1024);
    pipeline = pipeline.with_stale_days(stale_days);

    let mut candidates = pipeline.classify(&index);
    SafetyPipeline::evaluate_candidates(&mut candidates);

    let result = ScanResultJson::new(candidates, index.total_files_scanned, start_total.elapsed());

    if as_json {
        // 1. Progress event with ingestion metrics
        let prog = ScanStreamEvent::Progress {
            files_scanned: index.total_files_scanned,
            elapsed_ms: start_total.elapsed().as_millis(),
        };
        println!("{}", serde_json::to_string(&prog)?);

        // 2. Stream individual candidates progressively
        for candidate in &result.candidates {
            if !show_all && !candidate.is_safe {
                continue;
            }
            let cand_evt = ScanStreamEvent::Candidate {
                candidate: candidate.clone(),
            };
            println!("{}", serde_json::to_string(&cand_evt)?);
        }

        // 3. Final summary event
        let done_evt = ScanStreamEvent::Done { summary: result };
        println!("{}", serde_json::to_string(&done_evt)?);
    } else {
        print_human_table(&result, show_all);
    }

    Ok(())
}

fn run_clean(
    paths: &[PathBuf],
    permanent: bool,
    dry_run: bool,
    auto_confirm: bool,
    as_json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !as_json {
        println!();
        println!(
            "════════════════════════════════════════════════════════════════════════════════════════"
        );
        println!("  UNDRIFT CLEANUP REVIEW");
        println!(
            "════════════════════════════════════════════════════════════════════════════════════════"
        );
        let mode = if dry_run {
            "SIMULATION (Dry-Run)"
        } else if permanent {
            "PERMANENT DELETION"
        } else {
            "Recycle Bin (Default & Safe)"
        };
        println!("  Mode: {mode}");
        println!("  Items to remove: {}", paths.len());
        println!(
            "────────────────────────────────────────────────────────────────────────────────────────"
        );

        for (i, path) in paths.iter().enumerate() {
            println!("  [{}] {}", i + 1, path.display());
        }
        println!(
            "────────────────────────────────────────────────────────────────────────────────────────"
        );

        if !auto_confirm && !dry_run {
            print!("  Proceed with cleanup? [y/N]: ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let trimmed = input.trim().to_lowercase();
            if trimmed != "y" && trimmed != "yes" {
                println!("  Operation cancelled by user.");
                return Ok(());
            }
        }
    }

    let targets: Vec<(PathBuf, u64)> = paths
        .iter()
        .map(|p| {
            let size = if p.is_dir() {
                walkdir::WalkDir::new(p)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter_map(|e| e.metadata().ok())
                    .map(|m| m.len())
                    .sum()
            } else {
                std::fs::metadata(p).map(|m| m.len()).unwrap_or(0)
            };
            (p.clone(), size)
        })
        .collect();

    let report = CleanExecutor::clean_targets(&targets, permanent, dry_run);

    if as_json {
        let json_str = serde_json::to_string_pretty(&report)?;
        println!("{json_str}");
    } else {
        println!();
        println!(
            "  Cleanup complete! Successfully reclaimed: {}",
            report.human_total_reclaimed
        );
        if !report.failed.is_empty() {
            println!("  Encountered {} failure(s):", report.failed.len());
            for fail in &report.failed {
                println!("    - {}: {}", fail.path.display(), fail.error_message);
            }
        }
        println!();
    }

    Ok(())
}

fn run_history() {
    let records = HistoryManager::load_history();
    println!();
    println!(
        "════════════════════════════════════════════════════════════════════════════════════════"
    );
    println!("  UNDRIFT CLEANUP HISTORY");
    println!(
        "════════════════════════════════════════════════════════════════════════════════════════"
    );

    if records.is_empty() {
        println!("  No previous cleanup actions recorded.");
        println!(
            "────────────────────────────────────────────────────────────────────────────────────────"
        );
        return;
    }

    let total_reclaimed_all_time: u64 = records.iter().map(|r| r.reclaimed_bytes).sum();
    println!(
        "  Total cleanups: {}  |  Lifetime space reclaimed: {}",
        records.len(),
        undrift_core::model::candidate::format_size(total_reclaimed_all_time)
    );
    println!(
        "────────────────────────────────────────────────────────────────────────────────────────"
    );

    for (i, rec) in records.iter().enumerate() {
        let method = if rec.permanent {
            "Permanent"
        } else {
            "Recycle Bin"
        };
        println!(
            "  [{}] {} | Reclaimed: {} ({} items, {})",
            i + 1,
            rec.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            rec.human_reclaimed,
            rec.items_count,
            method
        );
        for p in &rec.paths {
            println!("      ↳ {p}");
        }
    }
    println!(
        "────────────────────────────────────────────────────────────────────────────────────────"
    );
    println!();
}
