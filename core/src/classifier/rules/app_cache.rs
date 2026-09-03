use super::ClassificationRule;
use crate::model::category::ArtifactCategory;
use crate::model::file_record::FileRecord;
use crate::scanner::ScanIndex;

/// Detects app-specific offline caches (Discord, Spotify, Teams, Slack, etc.).
///
/// These folders grow very large over time (Discord cache can reach 1–5 GB).
/// The apps re-populate them automatically on next launch. We do NOT touch
/// the user's actual app data, logs, or profile settings — only known cache
/// sub-directories within the LocalAppData tree.
pub struct AppCacheRule;

impl ClassificationRule for AppCacheRule {
    fn name(&self) -> &'static str {
        "App Junk & Offline Cache"
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

        // Discord: %LOCALAPPDATA%\Discord\Cache\ and %LOCALAPPDATA%\discord\GPUCache
        let is_discord_cache = (name == "cache" || name == "gpucache" || name == "code cache")
            && path_str.contains("\\discord\\");

        // Spotify: %LOCALAPPDATA%\Spotify\Storage\  and  \Data\ sub-folder
        let is_spotify_cache = (name == "storage" || name == "data")
            && path_str.contains("\\spotify\\")
            && !path_str.contains("\\spotify\\users\\"); // keep user playlists

        // Microsoft Teams: cache inside the app package folder
        let is_teams_cache = name == "cache"
            && (path_str.contains("\\microsoft\\teams\\")
                || path_str.contains("\\teamsupdater\\"));

        // Slack: %LOCALAPPDATA%\slack\Cache\
        let is_slack_cache =
            (name == "cache" || name == "gpucache") && path_str.contains("\\slack\\");

        // WhatsApp Desktop cache
        let is_whatsapp_cache = name == "cache" && path_str.contains("\\whatsapp\\");

        // Zoom: %APPDATA%\Zoom\data  and  %LOCALAPPDATA%\Zoom\
        let is_zoom_cache = name == "data" && path_str.contains("\\zoom\\");

        if is_discord_cache {
            Some((
                ArtifactCategory::AppCache,
                "Discord media cache — Discord will re-download images and videos as needed".to_string(),
            ))
        } else if is_spotify_cache {
            Some((
                ArtifactCategory::AppCache,
                "Spotify offline cache — songs will stream fresh from Spotify's servers".to_string(),
            ))
        } else if is_teams_cache {
            Some((
                ArtifactCategory::AppCache,
                "Microsoft Teams cache — Teams will rebuild it on next sign-in".to_string(),
            ))
        } else if is_slack_cache {
            Some((
                ArtifactCategory::AppCache,
                "Slack cache — Slack will re-download files and images as you use it".to_string(),
            ))
        } else if is_whatsapp_cache {
            Some((
                ArtifactCategory::AppCache,
                "WhatsApp Desktop cache — messages and media remain on your phone".to_string(),
            ))
        } else if is_zoom_cache {
            Some((
                ArtifactCategory::AppCache,
                "Zoom cache — your meetings and recordings are unaffected".to_string(),
            ))
        } else {
            None
        }
    }
}
