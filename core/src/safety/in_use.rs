use std::fs::OpenOptions;
use std::path::Path;

pub struct InUseChecker;

impl InUseChecker {
    pub fn is_locked(path: &Path) -> Option<String> {
        #[cfg(windows)]
        {
            if let Some(reason) = Self::check_restart_manager(path) {
                return Some(reason);
            }
        }

        Self::check_file_locks_heuristic(path)
    }

    #[cfg(windows)]
    fn check_restart_manager(path: &Path) -> Option<String> {
        use std::os::windows::ffi::OsStrExt;
        use windows::Win32::Foundation::{ERROR_MORE_DATA, WIN32_ERROR};
        use windows::Win32::System::RestartManager::{
            CCH_RM_SESSION_KEY, RM_PROCESS_INFO, RmEndSession, RmGetList, RmRegisterResources,
            RmStartSession,
        };
        use windows::core::{PCWSTR, PWSTR};

        let mut session_handle: u32 = 0;
        let mut session_key = [0u16; (CCH_RM_SESSION_KEY + 1) as usize];

        let err =
            unsafe { RmStartSession(&mut session_handle, None, PWSTR(session_key.as_mut_ptr())) };
        if err != WIN32_ERROR(0) {
            return None;
        }

        struct SessionGuard(u32);
        impl Drop for SessionGuard {
            fn drop(&mut self) {
                unsafe {
                    let _ = RmEndSession(self.0);
                }
            }
        }
        let _guard = SessionGuard(session_handle);

        let wide_path: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let files = [PCWSTR(wide_path.as_ptr())];

        let err = unsafe { RmRegisterResources(session_handle, Some(&files), None, None) };
        if err != WIN32_ERROR(0) {
            return None;
        }

        let mut n_proc_info_needed: u32 = 0;
        let mut n_proc_info: u32 = 0;
        let mut reboot_reasons: u32 = 0;

        let err = unsafe {
            RmGetList(
                session_handle,
                &mut n_proc_info_needed,
                &mut n_proc_info,
                None,
                &mut reboot_reasons,
            )
        };

        if (err == ERROR_MORE_DATA || err == WIN32_ERROR(0)) && n_proc_info_needed > 0 {
            let mut proc_info = vec![RM_PROCESS_INFO::default(); n_proc_info_needed as usize];
            n_proc_info = n_proc_info_needed;

            let err = unsafe {
                RmGetList(
                    session_handle,
                    &mut n_proc_info_needed,
                    &mut n_proc_info,
                    Some(proc_info.as_mut_ptr()),
                    &mut reboot_reasons,
                )
            };

            if err == WIN32_ERROR(0) && n_proc_info > 0 {
                let app_names: Vec<String> = proc_info
                    .iter()
                    .take(n_proc_info as usize)
                    .map(|info| {
                        let len = info
                            .strAppName
                            .iter()
                            .position(|&c| c == 0)
                            .unwrap_or(info.strAppName.len());
                        String::from_utf16_lossy(&info.strAppName[..len])
                    })
                    .filter(|s| !s.is_empty())
                    .collect();

                if !app_names.is_empty() {
                    return Some(format!(
                        "Active file locks held by: {}",
                        app_names.join(", ")
                    ));
                } else {
                    return Some(format!(
                        "Active file locks held by {} running process(es)",
                        n_proc_info
                    ));
                }
            }
        }

        None
    }

    fn check_file_locks_heuristic(path: &Path) -> Option<String> {
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
