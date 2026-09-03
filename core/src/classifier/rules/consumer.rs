use super::ClassificationRule;
use crate::model::category::ArtifactCategory;
use crate::model::file_record::FileRecord;
use crate::scanner::ScanIndex;

/// Detects game launcher caches from Steam, Epic Games, GOG Galaxy, and EA App.
///
/// These are shader pre-compilation caches, download staging folders, and log
/// archives. Deleting them causes games to re-build shader caches on first launch
/// (may cause a one-time stutter), but does NOT delete save games or progress.
pub struct GameCacheRule;

impl ClassificationRule for GameCacheRule {
    fn name(&self) -> &'static str {
        "Game Launcher Cache"
    }

    fn evaluate(
        &self,
        _index: &ScanIndex,
        record: &FileRecord,
    ) -> Option<(ArtifactCategory, String)> {
        if !record.is_dir {
            return None;
        }

        let name = record.name.to_lowercase();
        let path_str = record.path.to_string_lossy().to_lowercase();

        // Steam: downloading (staged downloads), shadercache
        let is_steam_download_cache = name == "downloading"
            && (path_str.contains("\\steam\\") || path_str.contains("\\steamapps\\"));

        let is_steam_shader_cache = name == "shadercache"
            && path_str.contains("\\steam\\");

        // Epic Games: pending downloads staging area
        let is_epic_cache = name == ".egstore"
            && (path_str.contains("\\epic games\\")
                || path_str.contains("\\epicgames\\"));

        // GOG Galaxy: delivery cache
        let is_gog_cache = name == "delivery"
            && path_str.contains("\\gog galaxy\\");

        // EA App / Origin: download cache
        let is_ea_cache = name == "__donotdelete"
            && (path_str.contains("\\origin\\") || path_str.contains("\\ea app\\"));

        // Common across launchers: shader cache named directories
        let is_generic_shader = (name == "shadercache" || name == "shader cache" || name == "dx12shadercache")
            && (path_str.contains("\\steam\\")
                || path_str.contains("\\epic games\\")
                || path_str.contains("\\gog galaxy\\"));

        if is_steam_download_cache {
            Some((
                ArtifactCategory::GameCache,
                "Steam incomplete / staged downloads cache — safe to clear".to_string(),
            ))
        } else if is_steam_shader_cache || is_generic_shader {
            Some((
                ArtifactCategory::GameCache,
                "Game shader cache — games may take a bit longer to start once, then return to normal".to_string(),
            ))
        } else if is_epic_cache {
            Some((
                ArtifactCategory::GameCache,
                "Epic Games download staging folder — incomplete downloads, not installed games".to_string(),
            ))
        } else if is_gog_cache {
            Some((
                ArtifactCategory::GameCache,
                "GOG Galaxy delivery cache — your installed games and saves are unaffected".to_string(),
            ))
        } else if is_ea_cache {
            Some((
                ArtifactCategory::GameCache,
                "EA App download staging cache — installed games are untouched".to_string(),
            ))
        } else {
            None
        }
    }
}

/// Detects Windows Temp folder junk and crash dump files.
pub struct TempFilesRule;

impl ClassificationRule for TempFilesRule {
    fn name(&self) -> &'static str {
        "Temporary Files & Crash Dumps"
    }

    fn evaluate(
        &self,
        _index: &ScanIndex,
        record: &FileRecord,
    ) -> Option<(ArtifactCategory, String)> {
        let path_str = record.path.to_string_lossy().to_lowercase();

        // Windows %TEMP% and %TMP% directories (but not the root itself)
        let in_windows_temp = path_str.contains("\\appdata\\local\\temp\\")
            || path_str.contains("\\windows\\temp\\");

        if record.is_dir && in_windows_temp {
            let name = record.name.to_lowercase();
            // Skip if this looks like an in-use installer directory
            if name.starts_with("{") && name.ends_with("}") {
                // GUID-named directories may be active installs — skip
                return None;
            }
            return Some((
                ArtifactCategory::TempFiles,
                "Leftover temporary folder from Windows or an installer — safe to remove".to_string(),
            ));
        }

        // Crash dumps: .dmp files in WER (Windows Error Reporting) folders
        if !record.is_dir {
            let ext = std::path::Path::new(&record.name)
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.to_lowercase());

            if let Some(ext) = ext
                && ext == "dmp" {
                    return Some((
                        ArtifactCategory::CrashDumps,
                        "Windows crash dump file from a previous app crash — safe to delete".to_string(),
                    ));
                }

        }

        None
    }
}
