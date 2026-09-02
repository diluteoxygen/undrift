using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace Undrift.App.CoreInterop;

public sealed record ScanResult
{
    [JsonPropertyName("total_files_scanned")]
    public int TotalFilesScanned { get; init; }

    [JsonPropertyName("scan_time_ms")]
    public long ScanTimeMs { get; init; }

    [JsonPropertyName("total_reclaimable_bytes")]
    public ulong TotalReclaimableBytes { get; init; }

    [JsonPropertyName("human_total_reclaimable")]
    public string HumanTotalReclaimable { get; init; } = "0 B";

    [JsonPropertyName("safe_count")]
    public int SafeCount { get; init; }

    [JsonPropertyName("unsafe_count")]
    public int UnsafeCount { get; init; }

    [JsonPropertyName("candidates")]
    public List<CandidateItem> Candidates { get; init; } = [];
}

public sealed record CandidateItem
{
    [JsonPropertyName("id")]
    public string Id { get; init; } = string.Empty;

    [JsonPropertyName("category")]
    public string Category { get; init; } = string.Empty;

    [JsonPropertyName("path")]
    public string Path { get; init; } = string.Empty;

    [JsonPropertyName("display_path")]
    public string DisplayPath { get; init; } = string.Empty;

    [JsonPropertyName("size_bytes")]
    public ulong SizeBytes { get; init; }

    [JsonPropertyName("human_size")]
    public string HumanSize { get; init; } = string.Empty;

    [JsonPropertyName("last_modified")]
    public DateTimeOffset? LastModified { get; init; }

    [JsonPropertyName("file_count")]
    public ulong FileCount { get; init; }

    [JsonPropertyName("is_safe")]
    public bool IsSafe { get; init; }

    [JsonPropertyName("safety_reason")]
    public string SafetyReason { get; init; } = string.Empty;

    [JsonPropertyName("git_status")]
    public GitStatusItem? GitStatus { get; init; }

    [JsonPropertyName("default_selected")]
    public bool DefaultSelected { get; init; }
}

public sealed record GitStatusItem
{
    [JsonPropertyName("type")]
    public string Type { get; init; } = string.Empty;

    [JsonPropertyName("modified_count")]
    public int? ModifiedCount { get; init; }

    [JsonPropertyName("message")]
    public string? Message { get; init; }
}

public sealed record CleanRequest
{
    [JsonPropertyName("targets")]
    public List<CleanTarget> Targets { get; init; } = [];

    [JsonPropertyName("permanent")]
    public bool Permanent { get; init; }

    [JsonPropertyName("dry_run")]
    public bool DryRun { get; init; }
}

public sealed record CleanTarget
{
    [JsonPropertyName("path")]
    public string Path { get; init; } = string.Empty;

    [JsonPropertyName("size_bytes")]
    public ulong SizeBytes { get; init; }
}

public sealed record CleanReport
{
    [JsonPropertyName("total_reclaimed_bytes")]
    public ulong TotalReclaimedBytes { get; init; }

    [JsonPropertyName("human_total_reclaimed")]
    public string HumanTotalReclaimed { get; init; } = string.Empty;

    [JsonPropertyName("succeeded")]
    public List<CleanSuccessItem> Succeeded { get; init; } = [];

    [JsonPropertyName("failed")]
    public List<CleanFailureItem> Failed { get; init; } = [];

    [JsonPropertyName("is_dry_run")]
    public bool IsDryRun { get; init; }

    [JsonPropertyName("was_permanent")]
    public bool WasPermanent { get; init; }
}

public sealed record CleanSuccessItem
{
    [JsonPropertyName("path")]
    public string Path { get; init; } = string.Empty;

    [JsonPropertyName("bytes_reclaimed")]
    public ulong BytesReclaimed { get; init; }
}

public sealed record CleanFailureItem
{
    [JsonPropertyName("path")]
    public string Path { get; init; } = string.Empty;

    [JsonPropertyName("error_message")]
    public string ErrorMessage { get; init; } = string.Empty;
}
