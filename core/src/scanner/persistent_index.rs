use crate::model::file_record::FileRecord;
use crate::scanner::ScanIndex;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IncrementalIndexError {
    #[error("Index not found at: {0}")]
    NotFound(PathBuf),
    #[error(
        "USN Journal wrapped or reset: saved cursor {saved_usn} < lowest valid USN {lowest_usn}"
    )]
    JournalWrapped { saved_usn: i64, lowest_usn: i64 },
    #[error("USN Journal ID changed from {saved_id} to {current_id}")]
    JournalIdMismatch { saved_id: u64, current_id: u64 },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Unsupported platform for USN journal: {0}")]
    UnsupportedPlatform(String),
    #[error("Journal read error: {0}")]
    JournalError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentIndexMetadata {
    pub volume_id: String,
    pub usn_journal_id: u64,
    pub last_usn: i64,
    pub saved_at: DateTime<Utc>,
    pub total_records: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentIndex {
    pub metadata: PersistentIndexMetadata,
    pub records: Vec<FileRecord>,
}

#[derive(Debug, Clone, Copy)]
pub struct UsnCheckpoint {
    pub journal_id: u64,
    pub lowest_usn: i64,
    pub next_usn: i64,
}

impl PersistentIndex {
    pub fn new(volume_id: &str, usn_journal_id: u64, last_usn: i64, index: &ScanIndex) -> Self {
        let records = index.records.values().cloned().collect();
        Self {
            metadata: PersistentIndexMetadata {
                volume_id: volume_id.to_string(),
                usn_journal_id,
                last_usn,
                saved_at: Utc::now(),
                total_records: index.records.len(),
            },
            records,
        }
    }

    pub fn to_scan_index(&self, root_path: PathBuf) -> ScanIndex {
        let mut index = ScanIndex::new(root_path);
        for record in &self.records {
            index.insert_record(record.clone());
        }
        index
    }

    pub fn save_to_disk(&self, path: &Path) -> Result<(), IncrementalIndexError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, self)?;
        Ok(())
    }

    pub fn load_from_disk(path: &Path) -> Result<Self, IncrementalIndexError> {
        if !path.exists() {
            return Err(IncrementalIndexError::NotFound(path.to_path_buf()));
        }
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let index = serde_json::from_reader(reader)?;
        Ok(index)
    }

    /// Verifies if a saved cursor is still valid against the current volume USN journal state.
    /// Returns Ok(()) if delta read is valid, or Err(JournalWrapped / JournalIdMismatch) on wrap/reset.
    pub fn verify_journal_continuity(
        &self,
        checkpoint: &UsnCheckpoint,
    ) -> Result<(), IncrementalIndexError> {
        if checkpoint.journal_id != self.metadata.usn_journal_id {
            return Err(IncrementalIndexError::JournalIdMismatch {
                saved_id: self.metadata.usn_journal_id,
                current_id: checkpoint.journal_id,
            });
        }

        if self.metadata.last_usn < checkpoint.lowest_usn {
            return Err(IncrementalIndexError::JournalWrapped {
                saved_usn: self.metadata.last_usn,
                lowest_usn: checkpoint.lowest_usn,
            });
        }

        Ok(())
    }

    /// Applies USN deltas directly to an in-memory ScanIndex.
    pub fn apply_deltas(
        &mut self,
        index: &mut ScanIndex,
        deltas: &[UsnDeltaRecord],
        new_next_usn: i64,
    ) {
        for delta in deltas {
            match delta {
                UsnDeltaRecord::Created(rec) => {
                    index.insert_record(rec.clone());
                }
                UsnDeltaRecord::Deleted { id } => {
                    index.remove_record(*id);
                }
                UsnDeltaRecord::Renamed {
                    id,
                    new_parent_id,
                    new_name,
                } => {
                    index.update_record(*id, Some(new_name.clone()), Some(*new_parent_id), None);
                }
                UsnDeltaRecord::Modified { id, new_size } => {
                    index.update_record(*id, None, None, Some(*new_size));
                }
            }
        }

        self.metadata.last_usn = new_next_usn;
        self.metadata.saved_at = Utc::now();
        self.metadata.total_records = index.records.len();
        self.records = index.records.values().cloned().collect();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsnDeltaRecord {
    Created(FileRecord),
    Deleted {
        id: u64,
    },
    Renamed {
        id: u64,
        new_parent_id: u64,
        new_name: String,
    },
    Modified {
        id: u64,
        new_size: u64,
    },
}

pub fn get_default_index_dir() -> PathBuf {
    #[cfg(windows)]
    {
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            return PathBuf::from(local_app_data)
                .join("sweepie")
                .join("indexes");
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("sweepie")
            .join("indexes");
    }
    std::env::temp_dir().join("sweepie").join("indexes")
}

pub fn get_index_path_for_volume(volume: &str) -> PathBuf {
    let sanitized: String = volume
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    get_default_index_dir().join(format!("index_{sanitized}.json"))
}

#[cfg(windows)]
pub fn query_usn_checkpoint(drive_path: &Path) -> Result<UsnCheckpoint, IncrementalIndexError> {
    use std::ffi::CString;
    use std::mem::size_of;
    use std::os::raw::c_void;
    use windows::Win32::Foundation;
    use windows::Win32::Storage::FileSystem;
    use windows::Win32::System::IO;
    use windows::Win32::System::Ioctl;
    use windows::core::PCSTR;

    let path_str = drive_path.to_string_lossy().to_string();
    let volume_root = if path_str.starts_with(r"\\.\") {
        path_str
    } else {
        let drive_letter = path_str
            .chars()
            .find(|c| c.is_ascii_alphabetic())
            .unwrap_or('C');
        format!(r"\\.\{drive_letter}:")
    };

    let c_path = CString::new(volume_root)
        .map_err(|e| IncrementalIndexError::JournalError(format!("Invalid volume path: {e}")))?;

    let volume_handle = unsafe {
        FileSystem::CreateFileA(
            PCSTR::from_raw(c_path.as_bytes_with_nul().as_ptr()),
            (FileSystem::FILE_GENERIC_READ | FileSystem::FILE_GENERIC_WRITE).0,
            FileSystem::FILE_SHARE_READ
                | FileSystem::FILE_SHARE_WRITE
                | FileSystem::FILE_SHARE_DELETE,
            None,
            FileSystem::OPEN_EXISTING,
            FileSystem::FILE_FLAG_OVERLAPPED,
            None,
        )
    }
    .map_err(|e| IncrementalIndexError::JournalError(format!("Failed to open volume: {e}")))?;

    if volume_handle.is_invalid() {
        return Err(IncrementalIndexError::JournalError(
            "Failed to acquire volume handle for USN query".to_string(),
        ));
    }

    struct HandleGuard(Foundation::HANDLE);
    impl Drop for HandleGuard {
        fn drop(&mut self) {
            unsafe {
                let _ = Foundation::CloseHandle(self.0);
            }
        }
    }
    let _guard = HandleGuard(volume_handle);

    let mut journal_data = Ioctl::USN_JOURNAL_DATA_V2::default();
    let mut bytes_returned = 0u32;

    let ok = unsafe {
        IO::DeviceIoControl(
            volume_handle,
            Ioctl::FSCTL_QUERY_USN_JOURNAL,
            None,
            0,
            Some(&mut journal_data as *mut _ as *mut c_void),
            size_of::<Ioctl::USN_JOURNAL_DATA_V2>() as u32,
            Some(&mut bytes_returned),
            None,
        )
    };

    if ok.is_err() {
        return Err(IncrementalIndexError::JournalError(
            "FSCTL_QUERY_USN_JOURNAL failed".to_string(),
        ));
    }

    Ok(UsnCheckpoint {
        journal_id: journal_data.UsnJournalID,
        lowest_usn: journal_data.LowestValidUsn,
        next_usn: journal_data.NextUsn,
    })
}

#[cfg(not(windows))]
pub fn query_usn_checkpoint(_drive_path: &Path) -> Result<UsnCheckpoint, IncrementalIndexError> {
    Err(IncrementalIndexError::UnsupportedPlatform(
        "USN Change Journal is an NTFS/Windows feature".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_persistent_index_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("index_C.json");

        let mut scan_index = ScanIndex::new(PathBuf::from("/test/root"));
        scan_index.insert_record(FileRecord::new(
            10,
            0,
            "file.txt".to_string(),
            PathBuf::from("/test/root/file.txt"),
            false,
            2048,
            None,
            None,
            0,
        ));

        let persistent = PersistentIndex::new("C:", 12345, 1000, &scan_index);
        persistent.save_to_disk(&index_path).unwrap();

        let loaded = PersistentIndex::load_from_disk(&index_path).unwrap();
        assert_eq!(loaded.metadata.volume_id, "C:");
        assert_eq!(loaded.metadata.usn_journal_id, 12345);
        assert_eq!(loaded.metadata.last_usn, 1000);
        assert_eq!(loaded.records.len(), 1);
        assert_eq!(loaded.records[0].name, "file.txt");
        assert_eq!(loaded.records[0].size_bytes, 2048);
    }

    #[test]
    fn test_journal_wrap_detection() {
        let scan_index = ScanIndex::new(PathBuf::from("/test/root"));
        let persistent = PersistentIndex::new("C:", 9999, 500, &scan_index);

        // Case 1: Journal wrapped (saved 500 < lowest 1000)
        let wrapped_checkpoint = UsnCheckpoint {
            journal_id: 9999,
            lowest_usn: 1000,
            next_usn: 5000,
        };
        let err = persistent.verify_journal_continuity(&wrapped_checkpoint);
        assert!(matches!(
            err,
            Err(IncrementalIndexError::JournalWrapped { .. })
        ));

        // Case 2: Journal ID mismatch (journal deleted and recreated)
        let mismatch_checkpoint = UsnCheckpoint {
            journal_id: 8888,
            lowest_usn: 100,
            next_usn: 5000,
        };
        let err = persistent.verify_journal_continuity(&mismatch_checkpoint);
        assert!(matches!(
            err,
            Err(IncrementalIndexError::JournalIdMismatch { .. })
        ));

        // Case 3: Valid continuous journal
        let valid_checkpoint = UsnCheckpoint {
            journal_id: 9999,
            lowest_usn: 200,
            next_usn: 5000,
        };
        assert!(
            persistent
                .verify_journal_continuity(&valid_checkpoint)
                .is_ok()
        );
    }

    #[test]
    fn test_apply_usn_deltas_in_memory() {
        let mut scan_index = ScanIndex::new(PathBuf::from("/root"));
        scan_index.insert_record(FileRecord::new(
            1,
            0,
            "old.txt".to_string(),
            PathBuf::from("/root/old.txt"),
            false,
            100,
            None,
            None,
            0,
        ));

        let mut persistent = PersistentIndex::new("C:", 1, 100, &scan_index);

        let deltas = vec![
            // 1. Create new record
            UsnDeltaRecord::Created(FileRecord::new(
                2,
                0,
                "created.txt".to_string(),
                PathBuf::from("/root/created.txt"),
                false,
                500,
                None,
                None,
                0,
            )),
            // 2. Rename old.txt -> renamed.txt
            UsnDeltaRecord::Renamed {
                id: 1,
                new_parent_id: 0,
                new_name: "renamed.txt".to_string(),
            },
            // 3. Modify size of renamed.txt
            UsnDeltaRecord::Modified {
                id: 1,
                new_size: 999,
            },
        ];

        persistent.apply_deltas(&mut scan_index, &deltas, 250);

        assert_eq!(scan_index.records.len(), 2);
        assert_eq!(scan_index.get_record(1).unwrap().name, "renamed.txt");
        assert_eq!(scan_index.get_record(1).unwrap().size_bytes, 999);
        assert_eq!(scan_index.get_record(2).unwrap().name, "created.txt");
        assert_eq!(scan_index.get_record(2).unwrap().size_bytes, 500);
        assert_eq!(persistent.metadata.last_usn, 250);

        // Now test deletion delta
        let delete_deltas = vec![UsnDeltaRecord::Deleted { id: 1 }];
        persistent.apply_deltas(&mut scan_index, &delete_deltas, 300);

        assert_eq!(scan_index.records.len(), 1);
        assert!(scan_index.get_record(1).is_none());
        assert_eq!(scan_index.get_record(2).unwrap().name, "created.txt");
    }
}
