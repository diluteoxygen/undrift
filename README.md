<p align="center">
  <img src="app/Sweepie.App/Assets/AppIcon.png" width="120" alt="Sweepie icon">
</p>

<h1 align="center">Sweepie</h1>
<p align="center">Finds the disk space your build tools are quietly hoarding, and lets you take it back.</p>

<p align="center">
  <a href="https://github.com/diluteoxygen/undrift/actions/workflows/ci.yml"><img src="https://github.com/diluteoxygen/undrift/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
  <img src="https://img.shields.io/badge/platform-Windows%2010%2F11-0078D4.svg" alt="Windows 10/11">
</p>

---

Sweepie reads your NTFS Master File Table directly, so a full drive scan takes seconds, not minutes. It knows what a dev machine actually accumulates (`node_modules`, `target/`, `.venv`, IDE caches, dead installers) and only flags what it can prove is safe to remove. Nothing gets deleted without a review pass, and Recycle Bin is always the default.

Rust core, native WinUI 3 shell in C#/.NET. No Electron, no telemetry, no background service.

## What it finds

| Category | How it's detected | Why it's safe to remove |
|---|---|---|
| Node.js | `node_modules` next to `package.json` | Reinstalled with `npm`/`pnpm`/`yarn` |
| Rust | `target/` next to `Cargo.toml` | Rebuilt with `cargo build` |
| Python | `.venv` / `venv` / `env` next to a project manifest | Recreated on demand |
| Python bytecode | `__pycache__` | Regenerated automatically |
| .NET | `bin/` and `obj/` next to a `.csproj`/`.fsproj` | Rebuilt with `dotnet build` |
| Visual Studio | `.vs/` next to a `.sln` | Just an IntelliSense/solution cache |
| Gradle | `.gradle/` | Redownloaded on next build |
| Maven | `.m2/repository` | Redownloaded on next build |
| NuGet | `.nuget/packages` | Restored with `dotnet restore` |
| JetBrains | `.idea/` | IDE workspace index |
| Unity | `Library/` and `Temp/` | Rebuilt when the project reopens |
| Downloads | Installers (`.exe`/`.msi`/`.iso`) older than 30 days | Just old, one-off installers |
| Windows Update | `Windows.old` | Leftover previous-version backup |

## How it decides what's safe

Nothing gets offered for removal unless it clears all of this:

- The enclosing Git repo, if any, has no uncommitted changes
- It isn't a OneDrive cloud placeholder (never triggers a redownload or a desync)
- It isn't currently held open by another process
- It isn't a symlink or junction pointing somewhere unexpected

The same checks run again immediately before deletion, not just at scan time, since state can change in between. Recycle Bin is the default action. Permanent deletion is an explicit opt-in, every time.

## Fast the first time, faster after that

The first scan of a volume reads the MFT directly. Every scan after that reads only what changed since, via the NTFS USN journal, instead of rescanning from zero, so a second run reflects the current state of your disk almost instantly. If the journal's been rotated past what Sweepie last saw, it falls back to a full rescan automatically instead of working off a stale picture.

## What this deliberately isn't

No registry cleaner, no driver updater, no bundled toolbars, no subscription nags. No telemetry and no network calls beyond an explicit update check, and nothing runs as a background service. No score, no currency, no gamification. These are excluded on purpose, not missing by accident.

## Architecture

```
undrift/
├── core/                          # Rust engine: cdylib + CLI binary
│   ├── src/
│   │   ├── scanner/                # MFT scanner, USN-journal incremental index, dir-walk fallback
│   │   ├── classifier/              # Per-ecosystem detection rules
│   │   ├── safety/                  # Git-dirty, cloud-placeholder, in-use, symlink checks
│   │   ├── cleaner/                  # Recycle Bin executor + history log
│   │   ├── ffi.rs                     # C ABI for the WinUI 3 shell
│   │   └── main.rs                     # `sweepie` CLI
│   └── tests/
├── app/                            # C# / .NET 8 + WinUI 3 shell
│   └── Sweepie.App/
│       ├── CoreInterop/              # Streaming bridge into the core
│       ├── ViewModels/
│       └── MainWindow.xaml            # Fluent 2 / Mica
└── .github/workflows/              # cargo clippy -D warnings, cargo test, on every push
```

## CLI

```bash
# Human-readable table
sweepie scan C:

# Streams scan progress, then each finding, then a final summary (one JSON object per line)
sweepie scan C: --json

# Include items that were skipped, with the reason why
sweepie scan C: --all

# Move to Recycle Bin (default, safe)
sweepie clean "C:\dev\my-app\target" "C:\dev\web\node_modules"

# Preview without deleting anything
sweepie clean "C:\dev\my-app\target" --dry-run

# Permanent deletion has to be asked for explicitly
sweepie clean "C:\dev\my-app\target" --permanent

# What's been cleaned before, and how much space it recovered
sweepie history
```

## Building

```bash
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo build --release

dotnet build app/Sweepie.App.sln
```
