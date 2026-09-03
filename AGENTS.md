# AGENTS.md — Undrift Repository Guidelines

> Instructions for autonomous AI agents (Claude Code, Cursor, Copilot, Antigravity, Windsurf) contributing to Undrift.

## 1. What This Repository Is

Undrift is a **Windows-native disk space reclaiming instrument** for developers and power users.
- **Core Engine (`/core`)**: Pure Rust library (`cdylib`) and CLI (`undrift`). Enumerates NTFS volumes at MFT speed (via `ntfs-reader` / `usn-journal-rs`), classifies developer build artifacts, enforces multi-point safety checks (Git working tree clean check, cloud placeholders, active file locks), and moves targets to the Recycle Bin by default.
- **UI Shell (`/app`)**: Native C# / .NET 8 + WinUI 3 interface with Mica material, Segoe UI Variable typography, progressive scan streaming, and review-before-delete dialogs.

## 2. Context Pointers (Progressive Disclosure)

When working on specific domains, consult the authoritative documentation in `docs/context/`:
- **Architecture & Domain Model**: [`docs/context/CONTEXT.md`](docs/context/CONTEXT.md) — Deep module boundaries, data structures, and invariants.
- **Project Brief & Motivation**: [`docs/context/PROJECT_BRIEF.md`](docs/context/PROJECT_BRIEF.md) — Original specification and competitive analysis vs CCleaner / WizTree.
- **Feature Roadmap**: [`docs/context/ROADMAP.md`](docs/context/ROADMAP.md) — v1 MVP, v1.1 container reclaim, v2 treemap visualization.
- **Windows Setup**: [`docs/context/WINDOWS_SETUP.md`](docs/context/WINDOWS_SETUP.md) — Toolchains, Visual Studio workloads, and native execution.

## 3. Strict Non-Negotiable Invariants

1. **Safety First**: Never offer an item for deletion without running the Git-dirty safety check. If the enclosing Git repo has uncommitted changes, mark `is_safe: false` and `default_selected: false`.
2. **Recycle Bin Default**: Always use system Recycle Bin (`trash` crate / Windows Shell `SHFileOperationW`) as the default deletion action. Permanent deletion must always require an explicit user opt-in (`--permanent`).
3. **No Telemetry**: Zero analytics, zero background network calls (except explicit update checks), zero background services.
4. **Anti-Scope**: Never build registry cleaners, driver updaters, bundled toolbars, or gamified counters.
5. **Zero Warnings Policy**: All code must pass `cargo clippy --all-targets -- -D warnings` and `cargo test`.

## 4. Build, Test, and Lint Commands

```bash
# Run all tests
cargo test --all-targets

# Run strict clippy (-D warnings)
cargo clippy --all-targets -- -D warnings

# Check code formatting
cargo fmt --all --check

# Run the CLI scanner locally
cargo run -- scan . --all

# Run the CLI with JSON output
cargo run -- scan . --json

# Build release binaries
cargo build --release

# Build WinUI 3 application (.NET 8 on Windows)
dotnet build app/Undrift.App/Undrift.App.csproj
```

## 5. Coding Taste Profile (Matt Pocock Skills)

The repository incorporates Matt Pocock's skills in `.agents/skills/`:
- Use `codebase-design` to maintain deep modules with small interfaces and rich functionality.
- Use `tdd` to write red-green tests before adding new classifiers or safety rules.
- Use `unslop` to eliminate boilerplate, artificial comments, and generic filler.
- Use `diagnosing-bugs` for systematic reproduction loops.
