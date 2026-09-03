use crate::model::file_record::FileRecord;
use chrono::{DateTime, Utc};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::time::Duration;
use thiserror::Error;

#[cfg(windows)]
pub mod ntfs_mft;

pub mod dir_walk;

#[derive(Error, Debug)]
pub enum ScanError {
    #[error("I/O error during scan: {0}")]
    Io(#[from] std::io::Error),

    #[error("Administrative privileges required for raw MFT enumeration")]
    ElevationRequired,

    #[error("Volume or path not found: {0}")]
    NotFound(String),

    #[error("Scanning failure: {0}")]
    Other(String),
}

#[derive(Debug, Clone)]
pub struct ScanIndex {
    pub root_path: PathBuf,
    pub records: HashMap<u64, FileRecord>,
    pub children_by_parent: HashMap<u64, Vec<u64>>,
    pub name_index: HashMap<u64, HashMap<String, u64>>,
    pub scan_duration: Duration,
    pub total_files_scanned: usize,
}

impl ScanIndex {
    pub fn new(root_path: PathBuf) -> Self {
        Self {
            root_path,
            records: HashMap::new(),
            children_by_parent: HashMap::new(),
            name_index: HashMap::new(),
            scan_duration: Duration::ZERO,
            total_files_scanned: 0,
        }
    }

    pub fn insert_record(&mut self, record: FileRecord) {
        let id = record.id;
        let parent_id = record.parent_id;
        let name_lower = record.name.to_lowercase();

        self.children_by_parent
            .entry(parent_id)
            .or_default()
            .push(id);
        self.name_index
            .entry(parent_id)
            .or_default()
            .insert(name_lower, id);

        self.records.insert(id, record);
        self.total_files_scanned += 1;
    }

    pub fn get_record(&self, id: u64) -> Option<&FileRecord> {
        self.records.get(&id)
    }

    pub fn get_child_by_name(&self, parent_id: u64, name: &str) -> Option<&FileRecord> {
        let name_lower = name.to_lowercase();
        let child_id = self.name_index.get(&parent_id)?.get(&name_lower)?;
        self.records.get(child_id)
    }

    pub fn has_child_with_name(&self, parent_id: u64, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        self.name_index
            .get(&parent_id)
            .is_some_and(|map| map.contains_key(&name_lower))
    }

    pub fn has_child_with_extension(&self, parent_id: u64, extension: &str) -> bool {
        let ext_lower = extension.to_lowercase();
        self.children_by_parent
            .get(&parent_id)
            .is_some_and(|children| {
                children.iter().any(|&child_id| {
                    self.records.get(&child_id).is_some_and(|child| {
                        Path::new(&child.name)
                            .extension()
                            .and_then(|s| s.to_str())
                            .is_some_and(|ext| ext.eq_ignore_ascii_case(&ext_lower))
                    })
                })
            })
    }

    /// Lazily resolves the full path of a record by climbing the parent chain to the root.
    /// Used for candidates surfacing, avoiding expensive path computation during ingestion.
    pub fn resolve_path(&self, record_id: u64) -> PathBuf {
        if let Some(record) = self.records.get(&record_id)
            && !record.path.as_os_str().is_empty()
        {
            return record.path.clone();
        }

        let mut components = Vec::new();
        let mut curr_id = record_id;

        while let Some(record) = self.records.get(&curr_id) {
            if !record.name.is_empty() && record.name != "." && record.name != "\\" {
                components.push(record.name.clone());
            }

            // Stop at root directory
            if record.parent_id == 0 || record.parent_id == curr_id || record.parent_id == 5 {
                break;
            }
            curr_id = record.parent_id;
        }

        let mut path = self.root_path.clone();
        for comp in components.into_iter().rev() {
            path.push(comp);
        }
        path
    }

    /// Calculate total bytes, file count, and latest modified timestamp for a subtree
    pub fn calculate_subtree_stats(
        &self,
        root_record_id: u64,
    ) -> (u64, u64, Option<DateTime<Utc>>) {
        let mut total_bytes = 0u64;
        let mut total_files = 0u64;
        let mut latest_modified: Option<DateTime<Utc>> = None;

        let mut queue = VecDeque::new();
        queue.push_back(root_record_id);

        while let Some(current_id) = queue.pop_front() {
            if let Some(record) = self.records.get(&current_id) {
                // Don't recurse into reparse points / junctions
                if current_id != root_record_id && record.is_reparse_point {
                    continue;
                }

                if !record.is_dir {
                    total_bytes += record.size_bytes;
                    total_files += 1;
                }

                if let Some(mod_time) = record.modified_at {
                    latest_modified = Some(match latest_modified {
                        Some(prev) => prev.max(mod_time),
                        None => mod_time,
                    });
                }

                if let Some(children) = self.children_by_parent.get(&current_id) {
                    for &child_id in children {
                        queue.push_back(child_id);
                    }
                }
            }
        }

        (total_bytes, total_files, latest_modified)
    }
}

pub trait VolumeScanner: Send + Sync {
    fn scan(&self, target_path: &Path) -> Result<ScanIndex, ScanError>;
}
