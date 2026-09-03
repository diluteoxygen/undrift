using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Undrift.App.CoreInterop;

namespace Undrift.App.ViewModels;

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
    private string _statusMessage = "Ready to scan.";

    [ObservableProperty]
    private string _totalReclaimableDisplay = "0 B";

    [ObservableProperty]
    private string _selectedReclaimableDisplay = "0 B";

    [ObservableProperty]
    private string _scanTimeDisplay = "0.00s";

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

    public ObservableCollection<string> AvailableCategories { get; } =
    [
        "All",
        "Node.js Dependencies",
        "Rust Build Output",
        "Python Virtualenv",
        "Visual Studio Artifacts",
        "JetBrains Cache",
        "Stale Installer",
    ];

    public ObservableCollection<CandidateViewModel> AllCandidates { get; } = [];
    public ObservableCollection<CandidateViewModel> FilteredCandidates { get; } = [];

    public bool HasResults => FilteredCandidates.Count > 0;
    public bool IsEmptyResults => FilteredCandidates.Count == 0;


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
        StatusMessage = $"Scanning {TargetPath}...";
        AllCandidates.Clear();
        FilteredCandidates.Clear();
        FilesAnalyzedCount = 0;
        TotalReclaimableDisplay = "0 B";

        var progress = new Progress<ScanProgressEvent>(p =>
        {
            FilesAnalyzedCount = p.FilesScanned;
            StatusMessage = $"Scanning Master File Table... {p.FilesScanned:N0} records analyzed";
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
            }

            if (App.Current?.DispatcherQueue != null)
            {
                App.Current.DispatcherQueue.TryEnqueue(AddCandidate);
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
            ScanTimeDisplay = $"{result.ScanTimeMs / 1000.0:F2}s";
            TotalReclaimableDisplay = result.HumanTotalReclaimable;

            ApplyFilter();
            UpdateSelectedMetrics();

            StatusMessage = $"Scan completed in {ScanTimeDisplay}. Discovered {result.Candidates.Count} artifact groups.";
        }
        catch (Exception ex)
        {
            HasError = true;
            ErrorMessage = $"Scan error: {ex.Message}";
            StatusMessage = $"Scan error: {ex.Message}";
        }
        finally
        {
            IsScanning = false;
            HasScanned = true;
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

        // Prompt user confirmation dialog with explicit target list and recycle bin setting
        if (ConfirmCleanCallback != null)
        {
            bool proceed = await ConfirmCleanCallback(selected);
            if (!proceed)
            {
                StatusMessage = "Cleanup cancelled by user.";
                return;
            }
        }

        IsCleaning = true;
        HasError = false;
        ErrorMessage = string.Empty;
        StatusMessage = $"Reclaiming space ({selected.Count} items)...";

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

            // Remove succeeded candidates
            var succeededPaths = new HashSet<string>(report.Succeeded.Select(s => s.Path));
            var toRemove = AllCandidates.Where(c => succeededPaths.Contains(c.Path)).ToList();
            foreach (var r in toRemove)
            {
                AllCandidates.Remove(r);
            }

            ApplyFilter();
            UpdateSelectedMetrics();

            if (report.Failed != null && report.Failed.Count > 0)
            {
                HasError = true;
                ErrorMessage = $"{report.Failed.Count} item(s) could not be cleaned (e.g. {report.Failed.First().ErrorMessage}).";
                StatusMessage = $"Reclaimed {report.HumanTotalReclaimed}! {toRemove.Count} item(s) cleaned, but {report.Failed.Count} item(s) failed.";
            }
            else
            {
                StatusMessage = $"Reclaimed {report.HumanTotalReclaimed}! {toRemove.Count} item(s) moved to {(UseRecycleBin ? "Recycle Bin" : "Permanently Deleted")}.";
            }
        }
        catch (Exception ex)
        {
            HasError = true;
            ErrorMessage = $"Cleanup error: {ex.Message}";
            StatusMessage = $"Cleanup error: {ex.Message}";
        }
        finally
        {
            IsCleaning = false;
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
