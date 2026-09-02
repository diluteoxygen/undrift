use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactCategory {
    NodeModules,
    RustTarget,
    PythonVenv,
    PythonCache,
    GradleCache,
    MavenCache,
    NugetCache,
    VisualStudio,
    JetBrains,
    Unity,
    StaleInstaller,
    WindowsUpdate,
}

impl ArtifactCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::NodeModules => "Node.js Dependencies",
            Self::RustTarget => "Rust Build Output",
            Self::PythonVenv => "Python Virtual Environment",
            Self::PythonCache => "Python Bytecode Cache",
            Self::GradleCache => "Gradle Cache",
            Self::MavenCache => "Maven Package Cache",
            Self::NugetCache => "NuGet Package Cache",
            Self::VisualStudio => "Visual Studio Artifacts",
            Self::JetBrains => "JetBrains IDE Cache",
            Self::Unity => "Unity Project Cache",
            Self::StaleInstaller => "Stale Installer / Download",
            Self::WindowsUpdate => "Windows Update Leftovers",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::NodeModules => "node_modules directory generated from package.json",
            Self::RustTarget => "target/ compilation directory generated from Cargo.toml",
            Self::PythonVenv => "Virtualenv folder next to requirements.txt or pyproject.toml",
            Self::PythonCache => "__pycache__ directories with compiled Python bytecode",
            Self::GradleCache => "Gradle build cache and dependency store",
            Self::MavenCache => "Maven local repository .m2 store",
            Self::NugetCache => "NuGet global package cache directory",
            Self::VisualStudio => "bin/, obj/, and .vs/ folders from Visual Studio solutions",
            Self::JetBrains => ".idea/ workspace indexing and cache directory",
            Self::Unity => "Unity Library/ and Temp/ build cache directories",
            Self::StaleInstaller => "Old .exe / .msi installer packages left in Downloads",
            Self::WindowsUpdate => "Windows.old upgrade backups or leftover patch archives",
        }
    }

    pub fn icon_glyph(&self) -> &'static str {
        match self {
            Self::NodeModules => "📦",
            Self::RustTarget => "🦀",
            Self::PythonVenv => "🐍",
            Self::PythonCache => "⚡",
            Self::GradleCache => "🐘",
            Self::MavenCache => "🪶",
            Self::NugetCache => "🔷",
            Self::VisualStudio => "🟣",
            Self::JetBrains => "🧠",
            Self::Unity => "🎮",
            Self::StaleInstaller => "💾",
            Self::WindowsUpdate => "🪟",
        }
    }
}
