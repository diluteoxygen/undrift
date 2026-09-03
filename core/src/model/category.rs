use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactCategory {
    // --- Consumer-first categories ---
    /// Old setup files left in Downloads (everyone has these)
    StaleInstaller,
    /// Windows Update package leftovers
    WindowsUpdate,
    /// Browser caches (Chrome, Edge, Firefox, Brave)
    BrowserCache,
    /// App caches (Discord, Spotify, Teams, Slack, etc.)
    AppCache,
    /// Windows Temp folder junk and crash dumps
    TempFiles,
    /// Game launcher caches (Steam, Epic, GOG)
    GameCache,
    /// Crash dumps & diagnostic logs
    CrashDumps,

    // --- Developer / power user categories ---
    /// Node.js node_modules
    NodeModules,
    /// Rust target/ build directory
    RustTarget,
    /// Python virtual environment
    PythonVenv,
    /// Python __pycache__
    PythonCache,
    /// Gradle build cache
    GradleCache,
    /// Maven .m2 local repo
    MavenCache,
    /// NuGet global package cache
    NugetCache,
    /// Visual Studio bin/obj/.vs
    VisualStudio,
    /// JetBrains .idea cache
    JetBrains,
    /// Unity Library/Temp
    Unity,
}

impl ArtifactCategory {
    /// Friendly, plain-English name shown in the primary UI.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::StaleInstaller  => "Old Setup Files",
            Self::WindowsUpdate   => "Windows Update Leftovers",
            Self::BrowserCache    => "Web Browser Cache",
            Self::AppCache        => "App Junk & Offline Cache",
            Self::TempFiles       => "Temporary Files",
            Self::GameCache       => "Game Launcher Cache",
            Self::CrashDumps      => "Crash Reports & Logs",
            Self::NodeModules     => "Node.js Project Files",
            Self::RustTarget      => "Rust Build Files",
            Self::PythonVenv      => "Python Project Environment",
            Self::PythonCache     => "Python Bytecode Cache",
            Self::GradleCache     => "Java/Android Build Cache",
            Self::MavenCache      => "Java Package Cache",
            Self::NugetCache      => ".NET Package Cache",
            Self::VisualStudio    => "Visual Studio Build Files",
            Self::JetBrains       => "JetBrains IDE Cache",
            Self::Unity           => "Unity Game Build Cache",
        }
    }

    /// One-sentence, jargon-free description for the detail view.
    pub fn description(&self) -> &'static str {
        match self {
            Self::StaleInstaller  => "Old .exe and .msi setup files in your Downloads folder that you probably already installed.",
            Self::WindowsUpdate   => "Leftover Windows update packages and old Windows backup folders that are safe to remove.",
            Self::BrowserCache    => "Temporary web page data saved by Chrome, Edge, Firefox, or Brave. Frees space without losing bookmarks or passwords.",
            Self::AppCache        => "Offline data and media caches from apps like Discord, Spotify, or Teams. The app will re-download what it needs.",
            Self::TempFiles       => "Files in the Windows Temp folder that programs left behind. All safe to delete.",
            Self::GameCache       => "Shader and download caches from Steam, Epic Games, or GOG. Games may take slightly longer to load once after cleaning.",
            Self::CrashDumps      => "Crash report files and diagnostic logs from previous app crashes. You can safely delete these.",
            Self::NodeModules     => "Re-downloadable JavaScript packages for software projects (node_modules). Run 'npm install' to restore.",
            Self::RustTarget      => "Compiled Rust build output in the target/ folder. Rebuilt automatically when you run 'cargo build'.",
            Self::PythonVenv      => "Python project environment folder. Recreate with 'python -m venv' if needed.",
            Self::PythonCache     => "__pycache__ folders with compiled Python files. Regenerated automatically on next run.",
            Self::GradleCache     => "Gradle build and dependency cache for Java or Android projects.",
            Self::MavenCache      => "Maven local repository (.m2) with downloaded Java libraries.",
            Self::NugetCache      => "NuGet global package cache. Packages are re-downloaded from the internet when needed.",
            Self::VisualStudio    => "Visual Studio build output (bin/, obj/) and IntelliSense index (.vs/). Rebuilt by Visual Studio.",
            Self::JetBrains       => "JetBrains IDE index and workspace cache (.idea/). Rebuilt when you reopen the project.",
            Self::Unity           => "Unity game engine build cache (Library/, Temp/). Regenerated on next Unity project open.",
        }
    }

    /// Whether this category is primarily aimed at general consumers (not developers).
    pub fn is_consumer(&self) -> bool {
        matches!(
            self,
            Self::StaleInstaller | Self::WindowsUpdate | Self::BrowserCache
                | Self::AppCache | Self::TempFiles | Self::GameCache | Self::CrashDumps
        )
    }

    pub fn icon_glyph(&self) -> &'static str {
        match self {
            Self::StaleInstaller  => "💾",
            Self::WindowsUpdate   => "🪟",
            Self::BrowserCache    => "🌐",
            Self::AppCache        => "📱",
            Self::TempFiles       => "🗑️",
            Self::GameCache       => "🎮",
            Self::CrashDumps      => "🐛",
            Self::NodeModules     => "📦",
            Self::RustTarget      => "🦀",
            Self::PythonVenv      => "🐍",
            Self::PythonCache     => "⚡",
            Self::GradleCache     => "🐘",
            Self::MavenCache      => "🪶",
            Self::NugetCache      => "🔷",
            Self::VisualStudio    => "🟣",
            Self::JetBrains       => "🧠",
            Self::Unity           => "🕹️",
        }
    }
}

