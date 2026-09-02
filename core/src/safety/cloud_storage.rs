#[cfg(windows)]
use crate::model::file_record::{
    FILE_ATTRIBUTE_OFFLINE, FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS, FILE_ATTRIBUTE_RECALL_ON_OPEN,
    FILE_ATTRIBUTE_REPARSE_POINT,
};
use std::path::Path;

pub struct CloudStorageChecker;

impl CloudStorageChecker {
    pub fn is_cloud_placeholder(path: &Path) -> Option<String> {
        let path_str = path.to_string_lossy().to_lowercase();

        // Path heuristic for common cloud storage folders
        let in_cloud_storage = path_str.contains("onedrive")
            || path_str.contains("icloud")
            || path_str.contains("dropbox")
            || path_str.contains("google drive");

        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if let Ok(metadata) = std::fs::symlink_metadata(path) {
                let attrs = metadata.file_attributes();
                let is_reparse = (attrs & FILE_ATTRIBUTE_REPARSE_POINT) != 0;
                let is_recall = (attrs
                    & (FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS
                        | FILE_ATTRIBUTE_RECALL_ON_OPEN
                        | FILE_ATTRIBUTE_OFFLINE))
                    != 0;

                if is_recall {
                    return Some(
                        "File is an online-only cloud placeholder (recall on access)".to_string(),
                    );
                }

                if is_reparse && in_cloud_storage {
                    return Some(
                        "Path is managed as a cloud storage sync reparse point".to_string(),
                    );
                }
            }
        }

        #[cfg(not(windows))]
        {
            if std::fs::symlink_metadata(path).is_ok_and(|m| m.is_symlink() && in_cloud_storage) {
                return Some("Path is managed as a cloud storage sync link".to_string());
            }
        }

        None
    }
}
