use super::ClassificationRule;
use crate::model::category::ArtifactCategory;
use crate::model::file_record::FileRecord;
use crate::scanner::ScanIndex;

/// Detects browser cache directories for Chrome, Edge, Firefox, and Brave.
///
/// These are always safe to delete — browsers rebuild them on next launch.
/// We never touch bookmarks, passwords, extensions, or profile data.
pub struct BrowserCacheRule;

impl ClassificationRule for BrowserCacheRule {
    fn name(&self) -> &'static str {
        "Web Browser Cache"
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

        // Only match known cache sub-folders, never profile roots.
        // Patterns derived from official Chromium / Firefox / Edge paths on Windows.
        let is_browser_cache = match name.as_str() {
            // Chromium-family "Cache" and "Code Cache"
            "cache" => {
                path_str.contains("\\google\\chrome\\")
                    || path_str.contains("\\microsoft\\edge\\")
                    || path_str.contains("\\brave\\brave-browser\\")
                    || path_str.contains("\\vivaldi\\")
                    || path_str.contains("\\chromium\\")
            }
            "code cache" => {
                path_str.contains("\\google\\chrome\\")
                    || path_str.contains("\\microsoft\\edge\\")
                    || path_str.contains("\\brave\\brave-browser\\")
            }
            "gpucache" | "shader cache" => {
                path_str.contains("\\google\\chrome\\")
                    || path_str.contains("\\microsoft\\edge\\")
                    || path_str.contains("\\brave\\")
            }
            // Firefox cache
            "cache2" => path_str.contains("\\mozilla\\firefox\\"),
            "startupCache" => path_str.contains("\\mozilla\\firefox\\"),
            _ => false,
        };

        if is_browser_cache {
            Some((
                ArtifactCategory::BrowserCache,
                "Browser cache folder — safe to clear, won't affect bookmarks or passwords".to_string(),

            ))
        } else {
            None
        }
    }
}
