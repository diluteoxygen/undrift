use super::{ScanError, ScanIndex, VolumeScanner};
use crate::model::file_record::{FILE_ATTRIBUTE_DIRECTORY, FileRecord};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;
use walkdir::WalkDir;

pub struct DirWalkScanner;

impl Default for DirWalkScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl DirWalkScanner {
    pub fn new() -> Self {
        Self
    }
}

impl VolumeScanner for DirWalkScanner {
    fn scan(&self, target_path: &Path) -> Result<ScanIndex, ScanError> {
        let start = Instant::now();
        let target_canonical = target_path
            .canonicalize()
            .unwrap_or_else(|_| target_path.to_path_buf());
        let mut index = ScanIndex::new(target_canonical.clone());

        let mut path_to_id: HashMap<PathBuf, u64> = HashMap::new();
        let mut next_id = 1u64;

        // Root entry
        path_to_id.insert(target_canonical.clone(), 0);
        let root_metadata = std::fs::metadata(&target_canonical).ok();
        let root_record = FileRecord::new(
            0,
            0,
            target_canonical
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string(),
            target_canonical.clone(),
            true,
            0,
            root_metadata
                .as_ref()
                .and_then(|m| m.modified().ok())
                .map(DateTime::<Utc>::from),
            root_metadata
                .as_ref()
                .and_then(|m| m.created().ok())
                .map(DateTime::<Utc>::from),
            FILE_ATTRIBUTE_DIRECTORY,
        );
        index.insert_record(root_record);

        for entry_res in WalkDir::new(&target_canonical).min_depth(1).into_iter() {
            let entry = match entry_res {
                Ok(e) => e,
                Err(_) => continue, // Skip unreadable or permission denied files safely
            };

            let path = entry.path().to_path_buf();
            let parent_path = match path.parent() {
                Some(p) => p.to_path_buf(),
                None => target_canonical.clone(),
            };

            let parent_id = *path_to_id.get(&parent_path).unwrap_or(&0);
            let current_id = next_id;
            next_id += 1;
            path_to_id.insert(path.clone(), current_id);

            let is_dir = entry.file_type().is_dir();
            let is_symlink = entry.file_type().is_symlink();
            let metadata = entry.metadata().ok();

            let size_bytes = if is_dir {
                0
            } else {
                metadata.as_ref().map(|m| m.len()).unwrap_or(0)
            };

            let modified_at = metadata
                .as_ref()
                .and_then(|m| m.modified().ok())
                .map(DateTime::<Utc>::from);
            let created_at = metadata
                .as_ref()
                .and_then(|m| m.created().ok())
                .map(DateTime::<Utc>::from);

            let mut attributes = 0u32;
            if is_dir {
                attributes |= FILE_ATTRIBUTE_DIRECTORY;
            }
            if is_symlink {
                attributes |= crate::model::file_record::FILE_ATTRIBUTE_REPARSE_POINT;
            }

            let file_name = entry.file_name().to_str().unwrap_or("").to_string();

            #[cfg(unix)]
            let hard_link_count = {
                use std::os::unix::fs::MetadataExt;
                metadata.as_ref().map(|m| m.nlink() as u32).unwrap_or(1)
            };

            #[cfg(windows)]
            let hard_link_count = {
                use std::os::windows::io::AsRawHandle;
                if !is_dir {
                    std::fs::File::open(&path)
                        .ok()
                        .and_then(|f| {
                            let mut info =
                                windows::Win32::Storage::FileSystem::BY_HANDLE_FILE_INFORMATION::default();
                            let handle = windows::Win32::Foundation::HANDLE(f.as_raw_handle() as _);
                            unsafe {
                                windows::Win32::Storage::FileSystem::GetFileInformationByHandle(
                                    handle, &mut info,
                                )
                                .ok()
                                .map(|_| info.nNumberOfLinks)
                            }
                        })
                        .unwrap_or(1)
                } else {
                    1
                }
            };

            #[cfg(not(any(unix, windows)))]
            let hard_link_count = 1u32;

            let record = FileRecord::new(
                current_id,
                parent_id,
                file_name,
                path,
                is_dir,
                size_bytes,
                modified_at,
                created_at,
                attributes,
            )
            .with_hard_links(hard_link_count);

            index.insert_record(record);
        }

        index.scan_duration = start.elapsed();
        Ok(index)
    }
}
