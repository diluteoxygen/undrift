# ROADMAP.md — Undrift Product Roadmap

> Structured milestone plan for Undrift from v1 MVP to v2 WizTree-class visualization.

## Milestone v1 (MVP) — Current Target

- [x] High-performance Rust core engine with NTFS MFT and fallback directory scanner
- [x] Core dev-artifact classification rules (`node_modules`, `target`, `.venv`, `.vs`, `bin`/`obj`, `Downloads` installers)
- [x] Multi-point safety pipeline (Git-dirty repository check, cloud reparse points, lock probes)
- [x] Recycle Bin by default cleanup executor and audit history logger
- [x] Zero-warning CI pipeline with strict clippy (`-D warnings`) and automated integration tests
- [x] Native C# / .NET 8 WinUI 3 Fluent 2 shell with Mica backdrop and Segoe UI Variable
- [ ] Windows hardware live MFT benchmarking & memory profile tuning (< 50MB idle RAM)
- [ ] Code signing pipeline (Azure Artifact Signing or standard OV cert)
- [ ] Winget package distribution manifest (`winget install undrift`)

## Milestone v1.1 — Containers & Multi-Drive

- [ ] **Multi-Drive Enumeration**: Sequential or parallel MFT reads across all mounted volumes (`C:`, `D:`, etc.)
- [ ] **WSL2 Distro VHDX Compaction**: Detect internal slack space in `%LOCALAPPDATA%\Packages\...\LocalState\ext4.vhdx` and offer safe compaction via `wsl --manage <distro> --compact`
- [ ] **Docker Engine / Builder Cache Reclaim**: Connect via local named pipe (`\\.\pipe\docker_engine`) to discover dangling build caches, unused layers, and stopped containers
- [ ] **Microsoft Store Channel**: Packaged MSIX distribution channel for zero-friction consumer installs

## Milestone v2 — Treemap Visualization

- [ ] **In-Memory Squarified Treemap**: Interactive treemap visualizer rendered directly from the already-loaded MFT `ScanIndex` without requiring a second disk scan
- [ ] **Giant / Stale File Finder**: Discover unindexed individual files > 1GB untouched for over 12 months across non-development directories

## Strict Anti-Scope (Permanently Excluded)

- ❌ No registry cleaners (CCleaner's primary failure mode)
- ❌ No system tweaks or winget app-installer dashboards (WinUtil's distinct domain)
- ❌ No driver updaters, bundled toolbars, or third-party offers
- ❌ No telemetry, background services, or network phoning
- ❌ No gamified currencies, counters, or sound effects
