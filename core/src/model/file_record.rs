use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const FILE_ATTRIBUTE_READONLY: u32 = 0x00000001;
pub const FILE_ATTRIBUTE_HIDDEN: u32 = 0x00000002;
pub const FILE_ATTRIBUTE_SYSTEM: u32 = 0x00000004;
pub const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x00000010;
pub const FILE_ATTRIBUTE_ARCHIVE: u32 = 0x00000020;
pub const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x00000400;
pub const FILE_ATTRIBUTE_OFFLINE: u32 = 0x00001000;
pub const FILE_ATTRIBUTE_RECALL_ON_OPEN: u32 = 0x00040000;
pub const FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS: u32 = 0x00400000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRecord {
    pub id: u64,
    pub parent_id: u64,
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size_bytes: u64,
    pub modified_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub attributes: u32,
    pub is_reparse_point: bool,
    pub is_cloud_placeholder: bool,
}

impl FileRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: u64,
        parent_id: u64,
        name: String,
        path: PathBuf,
        is_dir: bool,
        size_bytes: u64,
        modified_at: Option<DateTime<Utc>>,
        created_at: Option<DateTime<Utc>>,
        attributes: u32,
    ) -> Self {
        let is_reparse_point = (attributes & FILE_ATTRIBUTE_REPARSE_POINT) != 0;
        let is_cloud_placeholder = (attributes
            & (FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS
                | FILE_ATTRIBUTE_RECALL_ON_OPEN
                | FILE_ATTRIBUTE_OFFLINE))
            != 0;

        Self {
            id,
            parent_id,
            name,
            path,
            is_dir,
            size_bytes,
            modified_at,
            created_at,
            attributes,
            is_reparse_point,
            is_cloud_placeholder,
        }
    }
}
