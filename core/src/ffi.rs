#![allow(clippy::not_unsafe_ptr_arg_deref)]

use crate::cleaner::CleanExecutor;
use crate::output::ScanResultJson;
use crate::scanner::VolumeScanner;
use serde::Deserialize;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct CleanRequestJson {
    pub targets: Vec<CleanTargetItem>,
    pub permanent: bool,
    pub dry_run: bool,
}

#[derive(Deserialize)]
struct CleanTargetItem {
    pub path: String,
    pub size_bytes: u64,
}

/// Frees a C-string allocated by Rust
/// # Safety
/// Pointer must be a valid CString allocated by this library or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn undrift_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

/// Returns library version string
#[unsafe(no_mangle)]
pub extern "C" fn undrift_version() -> *mut c_char {
    CString::new(env!("CARGO_PKG_VERSION")).unwrap().into_raw()
}

/// Performs a scan on a given directory path and returns JSON results
/// # Safety
/// Pointers must be valid C-strings and valid output storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn undrift_scan_path(
    path_ptr: *const c_char,
    json_out: *mut *mut c_char,
) -> i32 {
    if path_ptr.is_null() || json_out.is_null() {
        return -1;
    }

    let c_str = unsafe { CStr::from_ptr(path_ptr) };
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    let target_path = Path::new(path_str);
    let scanner: Box<dyn VolumeScanner> = {
        #[cfg(windows)]
        {
            // If scanning a root volume, use MFT scanner; otherwise dir walk
            if path_str.len() <= 3 && path_str.contains(':') {
                Box::new(crate::scanner::ntfs_mft::NtfsMftScanner::new())
            } else {
                Box::new(crate::scanner::dir_walk::DirWalkScanner::new())
            }
        }
        #[cfg(not(windows))]
        {
            Box::new(crate::scanner::dir_walk::DirWalkScanner::new())
        }
    };

    let index = match scanner.scan(target_path) {
        Ok(idx) => idx,
        Err(e) => {
            let err_json = format!(r#"{{"error":"{}"}}"#, e);
            if let Ok(c_err) = CString::new(err_json) {
                unsafe {
                    *json_out = c_err.into_raw();
                }
            }
            return -3;
        }
    };

    let pipeline = crate::classifier::ClassifierPipeline::default();
    let mut candidates = pipeline.classify(&index);
    crate::safety::SafetyPipeline::evaluate_candidates(&mut candidates);

    let result = ScanResultJson::new(candidates, index.total_files_scanned, index.scan_duration);
    let json_bytes = match serde_json::to_string(&result) {
        Ok(j) => j,
        Err(_) => return -4,
    };

    match CString::new(json_bytes) {
        Ok(c_res) => {
            unsafe {
                *json_out = c_res.into_raw();
            }
            0
        }
        Err(_) => -5,
    }
}

/// Executes cleanup of selected candidates via JSON request
/// # Safety
/// Pointers must be valid C-strings and valid output storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn undrift_clean_json(
    request_ptr: *const c_char,
    result_out: *mut *mut c_char,
) -> i32 {
    if request_ptr.is_null() || result_out.is_null() {
        return -1;
    }

    let c_str = unsafe { CStr::from_ptr(request_ptr) };
    let req_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    let request: CleanRequestJson = match serde_json::from_str(req_str) {
        Ok(r) => r,
        Err(_) => return -3,
    };

    let targets: Vec<(PathBuf, u64)> = request
        .targets
        .into_iter()
        .map(|t| (PathBuf::from(t.path), t.size_bytes))
        .collect();

    let report = CleanExecutor::clean_targets(&targets, request.permanent, request.dry_run);
    let json_bytes = match serde_json::to_string(&report) {
        Ok(j) => j,
        Err(_) => return -4,
    };

    match CString::new(json_bytes) {
        Ok(c_res) => {
            unsafe {
                *result_out = c_res.into_raw();
            }
            0
        }
        Err(_) => -5,
    }
}
