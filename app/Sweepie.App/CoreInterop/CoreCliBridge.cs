using System;
using System.Diagnostics;
using System.IO;
using System.Text;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;

namespace Sweepie.App.CoreInterop;

public sealed class CoreCliBridge
{
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNameCaseInsensitive = true,
    };

    public string CoreExecutablePath { get; }

    public CoreCliBridge(string? customPath = null)
    {
        CoreExecutablePath = customPath ?? FindExecutablePath();
    }

    private static string FindExecutablePath()
    {
        string baseDir = AppContext.BaseDirectory;
        string exeName = OperatingSystem.IsWindows() ? "sweepie.exe" : "sweepie";

        string[] candidateLocations =
        [
            Path.Combine(baseDir, exeName),
            Path.Combine(baseDir, "..", "..", "..", "..", "target", "release", exeName),
            Path.Combine(baseDir, "..", "..", "..", "..", "target", "debug", exeName),
            Path.Combine(baseDir, "..", "target", "release", exeName),
            Path.Combine(baseDir, "..", "target", "debug", exeName),
        ];

        foreach (string path in candidateLocations)
        {
            string fullPath = Path.GetFullPath(path);
            if (File.Exists(fullPath))
            {
                return fullPath;
            }
        }

        // Fallback to expecting executable in PATH
        return exeName;
    }

    public async Task<ScanResult> ScanAsync(
        string targetPath,
        bool includeAll = true,
        IProgress<ScanProgressEvent>? progress = null,
        Action<CandidateItem>? onCandidateFound = null,
        CancellationToken ct = default)
    {
        string arguments = $"scan \"{targetPath}\" --json" + (includeAll ? " --all" : "");

        ProcessStartInfo psi = new()
        {
            FileName = CoreExecutablePath,
            Arguments = arguments,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            UseShellExecute = false,
            CreateNoWindow = true,
            StandardOutputEncoding = Encoding.UTF8,
            StandardErrorEncoding = Encoding.UTF8,
        };

        using Process process = new() { StartInfo = psi };
        process.Start();

        ScanResult? finalSummary = null;
        List<CandidateItem> collectedCandidates = [];

        Task<string> errorTask = process.StandardError.ReadToEndAsync(ct);

        while (await process.StandardOutput.ReadLineAsync(ct) is { } line)
        {
            if (string.IsNullOrWhiteSpace(line)) continue;

            try
            {
                using var doc = JsonDocument.Parse(line);
                if (doc.RootElement.TryGetProperty("type", out var typeProp))
                {
                    string? eventType = typeProp.GetString();
                    switch (eventType)
                    {
                        case "progress":
                            var progEvt = JsonSerializer.Deserialize<ScanProgressEvent>(line, JsonOptions);
                            if (progEvt != null)
                            {
                                progress?.Report(progEvt);
                            }
                            break;

                        case "candidate":
                            var candEvt = JsonSerializer.Deserialize<ScanCandidateEvent>(line, JsonOptions);
                            if (candEvt?.Candidate != null)
                            {
                                collectedCandidates.Add(candEvt.Candidate);
                                onCandidateFound?.Invoke(candEvt.Candidate);
                            }
                            break;

                        case "done":
                            var doneEvt = JsonSerializer.Deserialize<ScanDoneEvent>(line, JsonOptions);
                            if (doneEvt?.Summary != null)
                            {
                                finalSummary = doneEvt.Summary;
                            }
                            break;
                    }
                }
            }
            catch (JsonException)
            {
                // Skip malformed lines or non-JSON logs
            }
        }

        await process.WaitForExitAsync(ct);
        string error = await errorTask;

        if (process.ExitCode != 0)
        {
            throw new InvalidOperationException($"Core scan failed (exit code {process.ExitCode}): {error}");
        }

        if (finalSummary != null)
        {
            return finalSummary;
        }

        return new ScanResult
        {
            Candidates = collectedCandidates,
            TotalFilesScanned = collectedCandidates.Count,
        };
    }

    public async Task<CleanReport> CleanAsync(CleanRequest request, CancellationToken ct = default)
    {
        // Build CLI command
        StringBuilder sb = new();
        sb.Append("clean");
        if (request.Permanent) sb.Append(" --permanent");
        if (request.DryRun) sb.Append(" --dry-run");
        sb.Append(" --yes --json");

        foreach (CleanTarget target in request.Targets)
        {
            sb.Append($" \"{target.Path}\"");
        }

        ProcessStartInfo psi = new()
        {
            FileName = CoreExecutablePath,
            Arguments = sb.ToString(),
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            UseShellExecute = false,
            CreateNoWindow = true,
            StandardOutputEncoding = Encoding.UTF8,
        };

        using Process process = new() { StartInfo = psi };
        process.Start();

        Task<string> outputTask = process.StandardOutput.ReadToEndAsync(ct);
        Task<string> errorTask = process.StandardError.ReadToEndAsync(ct);

        await process.WaitForExitAsync(ct);

        string output = await outputTask;
        string error = await errorTask;

        if (process.ExitCode != 0)
        {
            throw new InvalidOperationException($"Core clean failed (exit code {process.ExitCode}): {error}");
        }

        return JsonSerializer.Deserialize<CleanReport>(output, JsonOptions)
            ?? throw new InvalidOperationException("Failed to deserialize clean report JSON");
    }

    public static string FormatSize(ulong bytes)
    {
        const ulong KB = 1024;
        const ulong MB = KB * 1024;
        const ulong GB = MB * 1024;
        const ulong TB = GB * 1024;

        if (bytes >= TB) return $"{bytes / (double)TB:F2} TB";
        if (bytes >= GB) return $"{bytes / (double)GB:F2} GB";
        if (bytes >= MB) return $"{bytes / (double)MB:F2} MB";
        if (bytes >= KB) return $"{bytes / (double)KB:F2} KB";
        return $"{bytes} B";
    }
}
