use super::{ScanError, ScanIndex, VolumeScanner};
use crate::model::file_record::FileRecord;
use chrono::{DateTime, Utc};
use ntfs_reader::mft::Mft;
use ntfs_reader::volume::Volume;
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

        for file in mft.files() {
            let record_id = file.number();
            let best_name = match file.get_best_file_name(&mft) {
                Some(n) => n,
                None => continue,
            };

            let name = best_name.to_string();
            let parent_id = best_name.parent();
            let is_dir = file.is_directory();

            let mut created_at = None;
            let mut modified_at = None;
            let mut size_bytes = 0u64;
            let mut attributes = best_name.header.file_attributes;

            for rec in mft.file_records(&file) {
                rec.attributes(|att| {
                    if att.header.type_id
                        == ntfs_reader::api::NtfsAttributeType::StandardInformation as u32
                    {
                        if let Some(stdinfo) = att.as_standard_info() {
                            let c_time = ntfs_reader::api::ntfs_to_unix_time(stdinfo.creation_time);
                            let m_time =
                                ntfs_reader::api::ntfs_to_unix_time(stdinfo.modification_time);
                            created_at = DateTime::<Utc>::from_timestamp(
                                c_time.unix_timestamp(),
                                c_time.nanosecond(),
                            );
                            modified_at = DateTime::<Utc>::from_timestamp(
                                m_time.unix_timestamp(),
                                m_time.nanosecond(),
                            );
                            attributes = stdinfo.file_attributes;
                        }
                    }

                    if att.header.type_id == ntfs_reader::api::NtfsAttributeType::Data as u32
                        && att.header.name_length == 0
                    {
                        if att.header.is_non_resident == 0 {
                            if let Some(header) = att.resident_header() {
                                size_bytes = header.value_length as u64;
                            }
                        } else if let Some(header) = att.nonresident_header() {
                            if header.lowest_vcn == 0 {
                                size_bytes = header.data_size;
                            }
                        }
                    }
                });
            }

            let record = FileRecord::new(
                record_id,
                parent_id,
                name,
                PathBuf::new(), // Path is resolved lazily for surfaced candidates only
                is_dir,
                size_bytes,
                modified_at,
                created_at,
                attributes,
            );

            index.insert_record(record);
        }

        index.scan_duration = start.elapsed();
        Ok(index)
    }
}
