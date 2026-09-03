# CLAUDE.md — Claude Code Developer Guide for Undrift

> Project handoff and instructions for Claude Code running on Windows or Linux.

## Project Overview

Undrift is a high-performance Windows space-reclaiming utility combining WizTree-class MFT scanning speed, developer artifact judgment (Node, Rust, Python, .NET, IDEs), strict safety verification (Git-dirty checks), and a native WinUI 3 Fluent 2 interface.

## Architecture

- **`core/`**: Rust crate producing:
  - `undrift` (CLI binary for standalone use and subprocess invocation)
  - `undrift_core` (`cdylib` / DLL exporting C-ABI functions in `core/src/ffi.rs`)
- **`app/`**: C# / .NET 8 WinUI 3 project (`Undrift.App`) invoking `core` via `CoreCliBridge` or `CoreNativeFfi`.
- **`.agents/skills/`**: Curated Matt Pocock skills defining the team's engineering standards (deep modules, TDD, unslop).

## Essential Commands

```bash
# Verify Rust engine
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all --check

# Test CLI scanning
cargo run -- scan C: --json
cargo run -- scan . --all

# Build WinUI 3 app (Windows)
dotnet build app/Undrift.App.sln
dotnet run --project app/Undrift.App/Undrift.App.csproj
```

## Critical Rules to Respect

1. **Recycle Bin is Default**: `trash::delete` must remain the default removal mechanism. Never make permanent delete the default.
2. **Git-Dirty Guardrail**: Any artifact inside a Git repo with uncommitted changes must be marked unsafe and deselected.
3. **No Slop / No Fluff**: Write crisp, functional code. Keep comments focused strictly on non-obvious invariants.
4. **Zero Warnings Bar**: Never commit code that triggers a clippy warning under `-D warnings`.

See [`docs/context/CONTEXT.md`](docs/context/CONTEXT.md) for full domain definitions and [`docs/context/WINDOWS_SETUP.md`](docs/context/WINDOWS_SETUP.md) for Windows configuration.
