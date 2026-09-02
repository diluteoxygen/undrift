use super::ClassificationRule;
use crate::model::category::ArtifactCategory;
use crate::model::file_record::FileRecord;
use crate::scanner::ScanIndex;

pub struct DotnetJavaRule;

impl ClassificationRule for DotnetJavaRule {
    fn name(&self) -> &'static str {
        ".NET, Java, and Maven Caches"
    }

    fn evaluate(
        &self,
        index: &ScanIndex,
        record: &FileRecord,
    ) -> Option<(ArtifactCategory, String)> {
        if !record.is_dir {
            return None;
        }

        let name = record.name.to_lowercase();

        // Visual Studio / .NET obj and bin directories
        if name == "obj" || name == "bin" {
            let parent_id = record.parent_id;
            if index.has_child_with_extension(parent_id, "csproj")
                || index.has_child_with_extension(parent_id, "fsproj")
                || index.has_child_with_extension(parent_id, "vbproj")
            {
                return Some((
                    ArtifactCategory::VisualStudio,
                    format!(
                        ".NET compilation intermediate '{name}/' directory (rebuilt via dotnet build)"
                    ),
                ));
            }
        }

        // Gradle build cache
        if name == ".gradle" {
            return Some((
                ArtifactCategory::GradleCache,
                "Gradle build cache and artifact store (re-downloadable via gradle build)"
                    .to_string(),
            ));
        }

        // Maven local cache
        let path_str = record.path.to_string_lossy().to_lowercase();
        if path_str.ends_with(".m2/repository") || path_str.ends_with(r".m2\repository") {
            return Some((
                ArtifactCategory::MavenCache,
                "Maven local dependency cache (re-downloadable via mvn compile)".to_string(),
            ));
        }

        // NuGet package cache
        if (name == "packages" && path_str.contains(".nuget"))
            || path_str.ends_with(".nuget/packages")
            || path_str.ends_with(r".nuget\packages")
        {
            return Some((
                ArtifactCategory::NugetCache,
                "NuGet global package cache (re-downloadable via dotnet restore)".to_string(),
            ));
        }

        None
    }
}
