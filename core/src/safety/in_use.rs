use std::fs::OpenOptions;
use std::path::Path;

pub struct InUseChecker;

impl InUseChecker {
    pub fn is_locked(path: &Path) -> Option<String> {
        if path.is_file() {
            // Attempt to open file in read/write mode to verify no exclusive write lock is held
            let err = OpenOptions::new().read(true).write(true).open(path).err();
            if err.is_some_and(|e| e.kind() == std::io::ErrorKind::PermissionDenied) {
                return Some("File is currently open or locked by another process".to_string());
            }
        } else if path.is_dir() {
            // Probe common locked files if present (e.g. .lock files, cargo .package-cache)
            let potential_locks = [
                path.join(".lock"),
                path.join(".package-cache"),
                path.join(".cargo-lock"),
            ];

            for lock_file in &potential_locks {
                if lock_file.exists() {
                    let err = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .open(lock_file)
                        .err();
                    if err.is_some_and(|e| e.kind() == std::io::ErrorKind::PermissionDenied) {
                        return Some(format!(
                            "Lock file '{}' is active in another running process",
                            lock_file.file_name().unwrap_or_default().to_string_lossy()
                        ));
                    }
                }
            }
        }

        None
    }
}
