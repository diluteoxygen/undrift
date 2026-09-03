using CommunityToolkit.Mvvm.ComponentModel;
using Undrift.App.CoreInterop;

namespace Undrift.App.ViewModels;

public partial class CandidateViewModel : ObservableObject
{
    public CandidateItem Model { get; }

    [ObservableProperty]
    private bool _isSelected;

    public CandidateViewModel(CandidateItem model)
    {
        Model = model;
        _isSelected = model.DefaultSelected;
    }

    public string Id => Model.Id;
    public string Path => Model.Path;
    public string DisplayPath => Model.DisplayPath;
    public ulong SizeBytes => Model.SizeBytes;
    public string HumanSize => Model.HumanSize;
    public ulong FileCount => Model.FileCount;
    public bool IsSafe => Model.IsSafe;
    public bool CanSelect => Model.IsSafe;
    public string SafetyReason => Model.SafetyReason;

    public string ModifiedDisplay => Model.LastModified.HasValue
        ? Model.LastModified.Value.LocalDateTime.ToString("yyyy-MM-dd HH:mm")
        : "Unknown";

    public string CategoryDisplayName => Model.Category switch
    {
        "node_modules" => "Node.js Dependencies",
        "rust_target" => "Rust Build Output",
        "python_venv" => "Python Virtualenv",
        "python_cache" => "Python Cache",
        "gradle_cache" => "Gradle Cache",
        "maven_cache" => "Maven Cache",
        "nuget_cache" => "NuGet Cache",
        "visual_studio" => "Visual Studio Artifacts",
        "jetbrains" => "JetBrains Cache",
        "unity" => "Unity Project Cache",
        "stale_installer" => "Stale Installer",
        "windows_update" => "Windows Update Leftovers",
        _ => Model.Category,
    };

    public string CategoryGlyph => Model.Category switch
    {
        "node_modules" => "📦",
        "rust_target" => "🦀",
        "python_venv" => "🐍",
        "python_cache" => "⚡",
        "gradle_cache" => "🐘",
        "maven_cache" => "🪶",
        "nuget_cache" => "🔷",
        "visual_studio" => "🟣",
        "jetbrains" => "🧠",
        "unity" => "🎮",
        "stale_installer" => "💾",
        "windows_update" => "🪟",
        _ => "📁",
    };

    public string SafetyBadgeText => IsSafe ? "Safe to Reclaim" : "Review / Skipped";
    public string SafetyBadgeBackground => IsSafe ? "#107C41" : "#D83B01";

    public bool HasHardlinks => Model.HasHardlinks;
    public string? SizeCaveat => Model.SizeCaveat;
}
