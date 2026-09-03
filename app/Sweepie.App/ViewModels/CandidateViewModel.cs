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
        ? $"{Model.LastModified.Value.LocalDateTime:MMM d, yyyy}"
        : "Unknown";

    /// Friendly, plain-English name shown in primary view.
    public string CategoryDisplayName => Model.Category switch
    {
        // Consumer-first
        "stale_installer"  => "Old Setup Files",
        "windows_update"   => "Windows Update Leftovers",
        "browser_cache"    => "Web Browser Cache",
        "app_cache"        => "App Junk & Offline Cache",
        "temp_files"       => "Temporary Files",
        "game_cache"       => "Game Launcher Cache",
        "crash_dumps"      => "Crash Reports & Logs",
        // Developer
        "node_modules"     => "Node.js Project Files",
        "rust_target"      => "Rust Build Files",
        "python_venv"      => "Python Project Environment",
        "python_cache"     => "Python Cache",
        "gradle_cache"     => "Java/Android Build Cache",
        "maven_cache"      => "Java Package Cache",
        "nuget_cache"      => ".NET Package Cache",
        "visual_studio"    => "Visual Studio Build Files",
        "jetbrains"        => "JetBrains IDE Cache",
        "unity"            => "Unity Game Build Cache",
        _                  => Model.Category,
    };

    /// True if this is a consumer-friendly (non-developer) category.
    public bool IsConsumerCategory => Model.Category is
        "stale_installer" or "windows_update" or "browser_cache" or
        "app_cache" or "temp_files" or "game_cache" or "crash_dumps";

    /// Group label for progressive disclosure (consumer vs developer section).
    public string GroupLabel => IsConsumerCategory ? "Everyday Junk" : "Developer & App Files";

    /// Segoe Fluent Icons glyph codes
    public string CategoryGlyph => Model.Category switch
    {
        // Consumer-first
        "stale_installer"  => "\uE896", // Download
        "windows_update"   => "\uE777", // Windows Update
        "browser_cache"    => "\uE12B", // World / Globe
        "app_cache"        => "\uE8F4", // Storage / Cloud
        "temp_files"       => "\uE74D", // Delete / Trash
        "game_cache"       => "\uE7FC", // Game controller
        "crash_dumps"      => "\uE7BA", // Warning / Bug
        // Developer
        "node_modules"     => "\uE8B7", // Package
        "rust_target"      => "\uE912", // Build / Tool
        "python_venv"      => "\uE770", // Branch / Env
        "python_cache"     => "\uE945", // Flash / Cache
        "gradle_cache"     => "\uE8F1", // Library / Box
        "maven_cache"      => "\uE8F1", // Library / Box
        "nuget_cache"      => "\uE71D", // Package
        "visual_studio"    => "\uE7C3", // Application
        "jetbrains"        => "\uE7C3", // Application
        "unity"            => "\uE7FC", // Game
        _                  => "\uE8B7", // Default
    };

    private static readonly Microsoft.UI.Xaml.Media.SolidColorBrush SafeBg
        = new(Windows.UI.Color.FromArgb(0x1F, 0x10, 0x7C, 0x41));
    private static readonly Microsoft.UI.Xaml.Media.SolidColorBrush UnsafeBg
        = new(Windows.UI.Color.FromArgb(0x1F, 0xD8, 0x3B, 0x01));
    private static readonly Microsoft.UI.Xaml.Media.SolidColorBrush SafeFg
        = new(Windows.UI.Color.FromArgb(0xFF, 0x10, 0x7C, 0x41));
    private static readonly Microsoft.UI.Xaml.Media.SolidColorBrush UnsafeFg
        = new(Windows.UI.Color.FromArgb(0xFF, 0xD8, 0x3B, 0x01));

    /// Friendly badge label — no jargon.
    public string SafetyBadgeText => IsSafe ? "✓ Safe to clean" : "⚠ Needs review";
    public Microsoft.UI.Xaml.Media.Brush SafetyBadgeBackground => IsSafe ? SafeBg : UnsafeBg;
    public Microsoft.UI.Xaml.Media.Brush SafetyBadgeForeground => IsSafe ? SafeFg : UnsafeFg;

    public bool HasHardlinks => Model.HasHardlinks;
    public string? SizeCaveat => Model.SizeCaveat;
}
