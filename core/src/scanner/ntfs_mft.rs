use super::{ScanError, ScanIndex, VolumeScanner};
use crate::model::file_record::FileRecord;
use chrono::{DateTime, Utc};
use ntfs_reader::file_info::{FileInfo, HashMapCache};
use ntfs_reader::mft::Mft;
use ntfs_reader::volume::Volume;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub struct NtfsMftScanner;

impl Default for NtfsMftScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl NtfsMftScanner {
    pub fn new() -> Self {
        Self
    }

    /// Resolves drive letter / volume path for ntfs-reader (e.g., "C:" -> r"\\.\C:")
    fn format_volume_path(path: &Path) -> String {
        let path_str = path.to_string_lossy();
        if path_str.len() >= 2 && path_str.as_bytes()[1] == b':' {
            let drive_letter = &path_str[0..2];
            format!(r"\\.\{drive_letter}")
        } else {
            path_str.to_string()
        }
    }
}

impl VolumeScanner for NtfsMftScanner {
    fn scan(&self, target_path: &Path) -> Result<ScanIndex, ScanError> {
        let start = Instant::now();
        let volume_spec = Self::format_volume_path(target_path);

        let volume = Volume::new(&volume_spec)
            .map_err(|e| ScanError::Other(format!("Failed to open volume '{volume_spec}': {e}")))?;

        let mft = Mft::new(volume).map_err(|e| {
            ScanError::Other(format!("Failed to parse MFT on '{volume_spec}': {e}"))
        })?;

        let root_canonical = target_path.to_path_buf();
        let mut index = ScanIndex::new(root_canonical);
        let mut path_to_id: HashMap<PathBuf, u64> = HashMap::new();
        let mut cache = HashMapCache::default();
        let mut next_id = 1u64;

        for file in mft.files() {
            let info = FileInfo::with_cache(&mft, &file, &mut cache);
            let path = info.path;
            if path.as_os_str().is_empty() {
                continue;
            }

            let parent_path = path.parent().map(|p| p.to_path_buf()).unwrap_or_default();
            let parent_id = if parent_path.as_os_str().is_empty() {
                0
            } else {
                *path_to_id.entry(parent_path.clone()).or_insert_with(|| {
                    let id = next_id;
                    next_id += 1;
                    id
                })
            };

            let current_id = *path_to_id.entry(path.clone()).or_insert_with(|| {
                let id = next_id;
                next_id += 1;
                id
            });

            let modified_at = info.modified.map(|dt| {
                DateTime::<Utc>::from_timestamp(dt.unix_timestamp(), dt.nanosecond())
                    .unwrap_or_default()
            });
            let created_at = info.created.map(|dt| {
                DateTime::<Utc>::from_timestamp(dt.unix_timestamp(), dt.nanosecond())
                    .unwrap_or_default()
            });

            let record = FileRecord::new(
                current_id,
                parent_id,
                info.name,
                path,
                info.is_directory,
                info.size,
                modified_at,
                created_at,
                info.file_attributes,
            );

            index.insert_record(record);
        }

        index.scan_duration = start.elapsed();
        Ok(index)
    }
}
