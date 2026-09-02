# Undrift

> A precision, Windows-native space-reclaiming instrument for developers and power users. Built with a high-performance Rust MFT core and a native Fluent 2 WinUI 3 interface.

[![CI](https://github.com/vikrant-singh/undrift/actions/workflows/ci.yml/badge.svg)](https://github.com/vikrant-singh/undrift/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust 2024](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org)
[![.NET 8](https://img.shields.io/badge/.NET-8.0-purple.svg)](https://dotnet.microsoft.com)

---

## The Pitch

Windows users typically juggle three imperfect utilities:
- **CCleaner**: Poisoned reputation (multiple supply-chain compromises, persistent telemetry, registry cleaner risks, aggressive subscription nagging).
- **Chris Titus's WinUtil**: An outstanding system-tweaks and software installer tool, but not an intelligent space analyzer.
- **WizTree / TreeSize**: Fast at *finding* large folders, but completely lacking developer judgment on what is safe to remove.

**Undrift** fills this vacuum:
1. **MFT-speed volume scanning**: Enumerates NTFS Master File Table records directly in single-digit seconds, just like WizTree.
2. **Developer-aware judgment**: Recognizes orphaned `node_modules`, stale `target/` folders, unused `.venv` environments, `.gradle` caches, and Visual Studio build artifacts.
3. **Safety guarantees**:
   - **Skips Git repositories with uncommitted changes** (prevents deleting build output from work-in-progress branches).
   - **Skips OneDrive / cloud placeholder files** (never triggers unwanted re-downloads or desyncs).
   - **Probes active file locks** (never deletes artifacts in use by active IDEs or compilers).
   - **Respects junctions and symlinks** (never recurses out of bounds).
4. **Native Fluent 2 Shell**: Built in C# with WinUI 3, Mica backdrop, and Segoe UI Variable typography.
5. **Strict Anti-Scope**:
   - ❌ No telemetry, no network calls (except explicit update checks).
   - ❌ No registry cleaners, driver updaters, or bundled toolbars.
   - ❌ No gamified currencies or counters.
   - ❌ **Nothing is deleted without explicit user review. Recycle Bin is always the default.**

---

## Architecture

```
undrift/
├── core/                     # Core Engine (Rust cdylib + CLI binary)
│   ├── src/
│   │   ├── scanner/          # NTFS MFT direct scanner + cross-platform fallback
│   │   ├── classifier/       # Rule pipeline (Node, Rust, Python, .NET, Java, IDE, etc.)
│   │   ├── safety/           # Safety pipeline (Git-dirty, OneDrive reparse, in-use locks)
│   │   ├── cleaner/          # Recycle Bin executor (trash) + history log
│   │   ├── ffi.rs            # C-ABI exported functions for WinUI 3 P/Invoke
│   │   ├── output.rs         # Human table & JSON formatters
│   │   └── main.rs           # CLI binary (`undrift`)
│   └── tests/                # Integration tests (classification, git-dirty, cleanup)
├── app/                      # UI Shell (C# / .NET 8 + WinUI 3)
│   ├── Undrift.App/
│   │   ├── CoreInterop/      # CLI subprocess bridge + native FFI bindings
│   │   ├── ViewModels/       # MVVM State (MainViewModel, CandidateViewModel)
│   │   ├── MainWindow.xaml   # Fluent 2 / Mica interface
│   │   └── app.manifest      # Per-Monitor V2 DPI and UTF-8 manifest
└── .github/workflows/        # CI with `cargo clippy -- -D warnings` and `cargo test`
```

---

## Supported Categories

| Category | Detection Rule | Safe Action |
|---|---|---|
| **Node.js** | `node_modules` adjacent to `package.json` | Re-installable via `npm`/`pnpm`/`yarn` |
| **Rust** | `target/` adjacent to `Cargo.toml` | Rebuilt via `cargo build` |
| **Python** | `.venv`, `venv`, `env` adjacent to project manifest | Re-creatable virtual environment |
| **Python Bytecode** | `__pycache__` directories | Regenerated automatically on execution |
| **.NET / C#** | `bin/` and `obj/` adjacent to `*.csproj`/`*.fsproj` | Rebuilt via `dotnet build` |
| **Visual Studio** | `.vs/` adjacent to `*.sln` | Solution index & IntelliSense cache |
| **Gradle** | `.gradle/` build cache directory | Re-downloadable build cache |
| **Maven** | `.m2/repository` local package cache | Re-downloadable artifact cache |
| **NuGet** | `.nuget/packages` global package store | Re-downloadable via `dotnet restore` |
| **JetBrains** | `.idea/` indexing directory | IDE project workspace index |
| **Unity** | `Library/` and `Temp/` project folders | Regenerated when opening project in Unity |
| **Installers** | `.exe`, `.msi`, `.iso` in `Downloads` > 30 days old | Stale downloaded installers |
| **Windows Update** | `Windows.old` upgrade archive | Prior Windows version backup |

---

## CLI Usage

### 1. Scan a volume or directory

```bash
# Human-readable table
undrift scan C:

# Include skipped / unsafe items with reasons
undrift scan C: --all

# Output structured JSON for automation or GUI
undrift scan C: --json

# Filter by minimum size (e.g. 50 MB)
undrift scan C: --min-size 50
```

### 2. Clean selected artifacts

```bash
# Move to Recycle Bin (safe, default)
undrift clean "C:\dev\my-app\target" "C:\dev\web\node_modules"

# Dry-run preview
undrift clean "C:\dev\my-app\target" --dry-run

# Permanent deletion (explicit opt-in)
undrift clean "C:\dev\my-app\target" --permanent
```

### 3. Review cleanup history

```bash
undrift history
```

---

## Development & CI

Run tests and strict clippy checks:

```bash
# Run tests
cargo test --all-targets

# Run strict clippy (-D warnings)
cargo clippy --all-targets -- -D warnings

# Build release binaries
cargo build --release
```
