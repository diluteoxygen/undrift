using CommunityToolkit.Mvvm.ComponentModel;
using Sweepie.App.CoreInterop;

namespace Sweepie.App.ViewModels;

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

    // Segoe Fluent Icons glyph codes
    public string CategoryGlyph => Model.Category switch
    {
        "node_modules" => "\uE8B7", // Package
        "rust_target" => "\uE912", // Build / Tool
        "python_venv" => "\uE770", // Branch / Env
        "python_cache" => "\uE945", // Flash / Cache
        "gradle_cache" => "\uE8F1", // Library / Box
        "maven_cache" => "\uE8F1", // Library / Box
        "nuget_cache" => "\uE71D", // Package
        "visual_studio" => "\uE7C3", // Application
        "jetbrains" => "\uE7C3", // Application
        "unity" => "\uE7FC", // Game
        "stale_installer" => "\uE896", // Download
        "windows_update" => "\uE777", // Windows Update
        _ => "\uE8B7", // Default
    };

    private static readonly Microsoft.UI.Xaml.Media.SolidColorBrush SafeBg = new(Windows.UI.Color.FromArgb(0x1F, 0x10, 0x7C, 0x41));
    private static readonly Microsoft.UI.Xaml.Media.SolidColorBrush UnsafeBg = new(Windows.UI.Color.FromArgb(0x1F, 0xD8, 0x3B, 0x01));
    private static readonly Microsoft.UI.Xaml.Media.SolidColorBrush SafeFg = new(Windows.UI.Color.FromArgb(0xFF, 0x10, 0x7C, 0x41));
    private static readonly Microsoft.UI.Xaml.Media.SolidColorBrush UnsafeFg = new(Windows.UI.Color.FromArgb(0xFF, 0xD8, 0x3B, 0x01));

    public string SafetyBadgeText => IsSafe ? "Safe to Reclaim" : "Review / Skipped";
    public Microsoft.UI.Xaml.Media.Brush SafetyBadgeBackground => IsSafe ? SafeBg : UnsafeBg;
    public Microsoft.UI.Xaml.Media.Brush SafetyBadgeForeground => IsSafe ? SafeFg : UnsafeFg;

    public bool HasHardlinks => Model.HasHardlinks;
    public string? SizeCaveat => Model.SizeCaveat;
}
