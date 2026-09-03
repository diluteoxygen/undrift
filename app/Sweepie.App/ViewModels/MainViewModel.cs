using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Microsoft.UI.Dispatching;
using Sweepie.App.CoreInterop;

namespace Sweepie.App.ViewModels;

public partial class MainViewModel : ObservableObject
{
    private readonly CoreCliBridge _bridge;

    [ObservableProperty]
    private string _targetPath = OperatingSystem.IsWindows() ? "C:\\" : ".";

    [ObservableProperty]
    private bool _isScanning;

    [ObservableProperty]
    private bool _isCleaning;

    [ObservableProperty]
    private string _statusMessage = "Ready to find extra space on your PC.";

    [ObservableProperty]
    private string _totalReclaimableDisplay = "0 B";

    [ObservableProperty]
    private string _selectedReclaimableDisplay = "0 B";

    [ObservableProperty]
    private string _scanTimeDisplay = "";

    [ObservableProperty]
    private int _filesAnalyzedCount;

    [ObservableProperty]
    private bool _useRecycleBin = true;

    [ObservableProperty]
    private string _selectedCategoryFilter = "All";

    [ObservableProperty]
    private bool _hasScanned;

    [ObservableProperty]
    private bool _hasError;

    [ObservableProperty]
    private string _errorMessage = string.Empty;

    public Func<List<CandidateViewModel>, Task<bool>>? ConfirmCleanCallback { get; set; }

    /// Filter categories matching the new friendly display names
    public ObservableCollection<string> AvailableCategories { get; } =
    [
        "All",
        "Old Setup Files",
        "Windows Update Leftovers",
        "Web Browser Cache",
        "App Junk & Offline Cache",
        "Temporary Files",
        "Game Launcher Cache",
        "Crash Reports & Logs",
        "Node.js Project Files",
        "Rust Build Files",
        "Python Project Environment",
        "Python Cache",
        "Java/Android Build Cache",
        "Java Package Cache",
        ".NET Package Cache",
        "Visual Studio Build Files",
        "JetBrains IDE Cache",
        "Unity Game Build Cache",
    ];

    public ObservableCollection<CandidateViewModel> AllCandidates { get; } = [];
    public ObservableCollection<CandidateViewModel> FilteredCandidates { get; } = [];

    public bool HasResults => FilteredCandidates.Count > 0;
    public bool IsEmptyResults => FilteredCandidates.Count == 0;

    /// True when a scan has run and found items
    public bool HasResultsAfterScan => HasScanned && HasResults;

    /// True when scanning is still in progress (for capybara animation state)
    public bool IsIdle => !IsScanning && !IsCleaning;

    /// Friendly sub-line for the hero card
    public string HeroSubtitle => HasScanned
        ? (HasResults
            ? $"Found {FilteredCandidates.Count} group{(FilteredCandidates.Count == 1 ? "" : "s")} of stuff you can clean"
            : "Your PC looks clean! 🎉")
        : "Sweepie checks for leftovers, junk, and old files — your photos and documents are always safe";

    public MainViewModel()
    {
        _bridge = new CoreCliBridge();
    }

    partial void OnSelectedCategoryFilterChanged(string value)
    {
        ApplyFilter();
    }

    [RelayCommand]
    public async Task StartScanAsync()
    {
        if (IsScanning || IsCleaning) return;

        IsScanning = true;
        HasScanned = false;
        StatusMessage = $"Looking through {TargetPath}...";
        AllCandidates.Clear();
        FilteredCandidates.Clear();
        FilesAnalyzedCount = 0;
        TotalReclaimableDisplay = "0 B";
        OnPropertyChanged(nameof(IsIdle));
        OnPropertyChanged(nameof(HeroSubtitle));

        var progress = new Progress<ScanProgressEvent>(p =>
        {
            FilesAnalyzedCount = p.FilesScanned;
            StatusMessage = $"Scanning... {p.FilesScanned:N0} files checked";
        });

        Action<CandidateItem> onCandidateFound = candidate =>
        {
            void AddCandidate()
            {
                var vm = new CandidateViewModel(candidate);
                vm.PropertyChanged += (_, e) =>
                {
                    if (e.PropertyName == nameof(CandidateViewModel.IsSelected))
                    {
                        UpdateSelectedMetrics();
                    }
                };
                AllCandidates.Add(vm);
                ApplyFilter();
                UpdateSelectedMetrics();
                OnPropertyChanged(nameof(HeroSubtitle));
            }

            DispatcherQueue? dispatcher = DispatcherQueue.GetForCurrentThread();
            if (dispatcher != null)
            {
                dispatcher.TryEnqueue(AddCandidate);
            }
            else
            {
                AddCandidate();
            }
        };

        HasError = false;
        ErrorMessage = string.Empty;

        try
        {
            ScanResult result = await _bridge.ScanAsync(
                TargetPath,
                includeAll: true,
                progress: progress,
                onCandidateFound: onCandidateFound);

            FilesAnalyzedCount = result.TotalFilesScanned;
            ScanTimeDisplay = $"{result.ScanTimeMs / 1000.0:F1}s";
            TotalReclaimableDisplay = result.HumanTotalReclaimable;

            ApplyFilter();
            UpdateSelectedMetrics();

            int count = result.Candidates.Count;
            StatusMessage = count > 0
                ? $"Done! Found {count} group{(count == 1 ? "" : "s")} in {ScanTimeDisplay} — {result.HumanTotalReclaimable} can be freed"
                : $"All clean! No junk found in {ScanTimeDisplay}.";
        }
        catch (Exception ex)
        {
            HasError = true;
            ErrorMessage = $"Something went wrong: {ex.Message}";
            StatusMessage = "Scan couldn't complete. Please try again.";
        }
        finally
        {
            IsScanning = false;
            HasScanned = true;
            OnPropertyChanged(nameof(IsIdle));
            OnPropertyChanged(nameof(HeroSubtitle));
            OnPropertyChanged(nameof(HasResultsAfterScan));
        }
    }

    [RelayCommand]
    public void SelectAllSafe()
    {
        foreach (var c in FilteredCandidates.Where(c => c.CanSelect))
        {
            c.IsSelected = true;
        }
        UpdateSelectedMetrics();
    }

    [RelayCommand]
    public void DeselectAll()
    {
        foreach (var c in FilteredCandidates)
        {
            c.IsSelected = false;
        }
        UpdateSelectedMetrics();
    }

    [RelayCommand]
    public async Task CleanSelectedAsync()
    {
        var selected = AllCandidates.Where(c => c.IsSelected).ToList();
        if (selected.Count == 0 || IsCleaning) return;

        if (ConfirmCleanCallback != null)
        {
            bool proceed = await ConfirmCleanCallback(selected);
            if (!proceed)
            {
                StatusMessage = "Nothing was deleted — your files are safe.";
                return;
            }
        }

        IsCleaning = true;
        HasError = false;
        ErrorMessage = string.Empty;
        StatusMessage = $"Cleaning up {selected.Count} item{(selected.Count == 1 ? "" : "s")}...";
        OnPropertyChanged(nameof(IsIdle));

        try
        {
            var request = new CleanRequest
            {
                Permanent = !UseRecycleBin,
                DryRun = false,
                Targets = selected.Select(s => new CleanTarget
                {
                    Path = s.Path,
                    SizeBytes = s.SizeBytes,
                }).ToList(),
            };

            CleanReport report = await _bridge.CleanAsync(request);

            var succeededPaths = new HashSet<string>(report.Succeeded.Select(s => s.Path));
            var toRemove = AllCandidates.Where(c => succeededPaths.Contains(c.Path)).ToList();
            foreach (var r in toRemove)
            {
                AllCandidates.Remove(r);
            }

            ApplyFilter();
            UpdateSelectedMetrics();
            OnPropertyChanged(nameof(HeroSubtitle));

            if (report.Failed != null && report.Failed.Count > 0)
            {
                HasError = true;
                ErrorMessage = $"{report.Failed.Count} item(s) couldn't be cleaned. They may be in use by another app.";
                StatusMessage = $"Freed {report.HumanTotalReclaimed}! {toRemove.Count} cleaned, {report.Failed.Count} skipped.";
            }
            else
            {
                string destination = UseRecycleBin ? "Recycle Bin" : "permanently deleted";
                StatusMessage = $"🎉 Freed {report.HumanTotalReclaimed}! {toRemove.Count} item{(toRemove.Count == 1 ? "" : "s")} sent to {destination}.";
            }
        }
        catch (Exception ex)
        {
            HasError = true;
            ErrorMessage = $"Cleanup ran into a problem: {ex.Message}";
            StatusMessage = "Couldn't finish cleaning. Please try again.";
        }
        finally
        {
            IsCleaning = false;
            OnPropertyChanged(nameof(IsIdle));
        }
    }

    private void ApplyFilter()
    {
        FilteredCandidates.Clear();
        foreach (var c in AllCandidates)
        {
            if (SelectedCategoryFilter == "All" || c.CategoryDisplayName == SelectedCategoryFilter)
            {
                FilteredCandidates.Add(c);
            }
        }
        OnPropertyChanged(nameof(HasResults));
        OnPropertyChanged(nameof(IsEmptyResults));
        OnPropertyChanged(nameof(HasResultsAfterScan));
        OnPropertyChanged(nameof(HeroSubtitle));
    }

    private void UpdateSelectedMetrics()
    {
        ulong bytes = 0;
        foreach (var c in AllCandidates.Where(c => c.IsSelected))
        {
            bytes += c.SizeBytes;
        }
        SelectedReclaimableDisplay = CoreCliBridge.FormatSize(bytes);
    }
}
