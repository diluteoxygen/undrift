use std::path::Path;

pub struct ReparseChecker;

impl ReparseChecker {
    pub fn is_junction_or_symlink(path: &Path) -> Option<String> {
        if let Ok(meta) = std::fs::symlink_metadata(path) {
            if meta.is_symlink() {
                return Some(
                    "Target is a symbolic link or junction point (skipping target traversal)"
                        .to_string(),
                );
            }

            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                let attrs = meta.file_attributes();
                if (attrs & crate::model::file_record::FILE_ATTRIBUTE_REPARSE_POINT) != 0 {
                    return Some("Target is an NTFS reparse point or volume junction".to_string());
                }
            }
        }
        None
    }
}
