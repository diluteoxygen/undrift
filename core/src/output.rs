use crate::model::candidate::{ReclaimCandidate, format_size};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResultJson {
    pub total_files_scanned: usize,
    pub scan_time_ms: u128,
    pub total_reclaimable_bytes: u64,
    pub human_total_reclaimable: String,
    pub safe_count: usize,
    pub unsafe_count: usize,
    pub candidates: Vec<ReclaimCandidate>,
}

impl ScanResultJson {
    pub fn new(
        candidates: Vec<ReclaimCandidate>,
        total_files_scanned: usize,
        scan_duration: Duration,
    ) -> Self {
        let mut total_reclaimable_bytes = 0u64;
        let mut safe_count = 0;
        let mut unsafe_count = 0;

        for c in &candidates {
            if c.is_safe {
                total_reclaimable_bytes += c.size_bytes;
                safe_count += 1;
            } else {
                unsafe_count += 1;
            }
        }

        Self {
            total_files_scanned,
            scan_time_ms: scan_duration.as_millis(),
            total_reclaimable_bytes,
            human_total_reclaimable: format_size(total_reclaimable_bytes),
            safe_count,
            unsafe_count,
            candidates,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ScanStreamEvent {
    #[serde(rename = "progress")]
    Progress {
        files_scanned: usize,
        elapsed_ms: u128,
    },
    #[serde(rename = "candidate")]
    Candidate { candidate: ReclaimCandidate },
    #[serde(rename = "done")]
    Done { summary: ScanResultJson },
}

pub fn print_human_table(result: &ScanResultJson, show_all: bool) {
    println!();
    println!(
        "═════════════════════════════════════════════════════════════════════════════════════════════════"
    );
    println!("  SWEEPIE — High-Performance Disk Space Reclaimer");
    println!(
        "═════════════════════════════════════════════════════════════════════════════════════════════════"
    );
    println!(
        "  Scan completed in: {:.2}s  |  Files analyzed: {}  |  Reclaimable space: {}",
        result.scan_time_ms as f64 / 1000.0,
        result.total_files_scanned,
        result.human_total_reclaimable
    );
    println!(
        "─────────────────────────────────────────────────────────────────────────────────────────────────"
    );

    let displayed_candidates: Vec<_> = result
        .candidates
        .iter()
        .filter(|c| show_all || c.is_safe)
        .collect();

    if displayed_candidates.is_empty() {
        println!("  No reclaimable build artifacts or dead caches found.");
        println!(
            "─────────────────────────────────────────────────────────────────────────────────────────────────"
        );
        return;
    }

    println!(
        "  {:<10} {:<24} {:<10} {:<12} {:<40}",
        "STATUS", "CATEGORY", "SIZE", "MODIFIED", "PATH"
    );
    println!(
        "  {:-<10} {:-<24} {:-<10} {:-<12} {:-<40}",
        "", "", "", "", ""
    );

    for c in &displayed_candidates {
        let status_tag = if c.is_safe { "[SAFE]" } else { "[SKIP]" };
        let modified_str = c
            .last_modified
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let truncated_path = if c.display_path.len() > 45 {
            format!("...{}", &c.display_path[c.display_path.len() - 42..])
        } else {
            c.display_path.clone()
        };

        println!(
            "  {:<10} {:<24} {:<10} {:<12} {}",
            status_tag,
            c.category.display_name(),
            c.human_size,
            modified_str,
            truncated_path
        );

        if !c.is_safe {
            println!("             ↳ Reason: {}", c.safety_reason);
        }
    }

    println!(
        "─────────────────────────────────────────────────────────────────────────────────────────────────"
    );
    println!(
        "  Found {} safe items ({}). Review carefully before cleaning.",
        result.safe_count, result.human_total_reclaimable
    );
    if result.unsafe_count > 0 && !show_all {
        println!(
            "  Note: {} item(s) skipped due to active locks or dirty git trees. Use --all to view.",
            result.unsafe_count
        );
    }
    println!();
}
