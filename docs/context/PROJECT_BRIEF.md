# Project Brief: A Windows-Native Space-Reclaiming Tool, Done Right

> This file is written as the handoff doc for Claude Code — everything it needs to start scaffolding without a back-and-forth.

## 0. The Reference Point

[chippytea](https://www.chippytea.com/) is a native macOS menu-bar app (SwiftUI front end, Rust core) that finds old build folders, dead dependencies, and stale installers, shows you what removing them costs and means, and lets you choose. No account, no telemetry, nothing deleted automatically. It also has a decorative "chip counter" game economy and a singalong song on the landing page — charming for a solo indie project's marketing, irrelevant to why the tool is actually good.

The ask: the Windows equivalent, minus the gimmick layer, executed to a higher technical and design bar than anything currently on Windows.

## 1. The Pitch, One Paragraph

Windows has three separate tools doing pieces of this badly: CCleaner (trusted brand, actively poisoned reputation), Chris Titus's WinUtil (excellent, but a system-config/app-installer kitchen sink — not a space tool), and WizTree/TreeSize (blazing fast at *finding* big things, zero judgment about what's safe to remove). Nothing on Windows combines MFT-speed scanning, developer-workflow-aware judgment about what's safe to delete, and a native Fluent 2 interface that doesn't look like it shipped in 2011. That's the gap.

## 2. The Competitive Map (Why Each Existing Option Isn't It)

**CCleaner** — the brand most people still reach for by reflex, and the one most worth displacing. Piriform sold to Avast in 2017; two months later CCleaner's official installer was compromised in a supply-chain attack that hit roughly 2.27 million users, and a second internal breach followed in 2019. Since then: persistent telemetry/"active monitoring" that can't be fully disabled, aggressive upsell nagging in the free tier, a registry cleaner that occasionally breaks things it shouldn't touch, and a $29.95/yr Pro tier for features Windows now does natively. By 2026 the mainstream tech press consensus is to actively steer people away from it. This is the trust vacuum to fill — explicitly, by not doing a single one of the things that put CCleaner here.

**Chris Titus's WinUtil** — genuinely great, and not a competitor because it's not the same tool. It's a PowerShell-driven installer/debloat/tweaks utility: bulk-install apps via winget, strip Windows telemetry and bloat, flip performance/privacy toggles, manage Windows Update behavior, build debloated ISOs. Six years, 200+ contributors, 30M+ runs, one of the most-starred PowerShell projects on GitHub. It does not analyze disk usage or clean dev caches. Copying its shape (a giant checklist of toggles) would be a mistake — the brief explicitly rules this out, correctly. Stay in a different lane entirely.

**WizTree / TreeSize / WinDirStat** — the closest thing to a technical benchmark to beat. WizTree reads the NTFS Master File Table directly instead of walking the directory tree, so a full 1TB NVMe scan finishes in seconds instead of minutes. That's the right scanning technique. But these tools stop at visualization: a treemap and a sortable list. No concept of "this is a stale node_modules," "this project hasn't been touched in 8 months," or "this is safe to delete without breaking your Git history." The judgment layer is entirely missing, and that's the harder, more valuable half of the problem.

**BleachBit / Microsoft PC Manager / Glary Utilities** — open-source or Microsoft-native, safe and inoffensive, and blind to developer workflows. None of them know what `target`, `.venv`, `node_modules`, Gradle/Maven caches, or a bloated WSL2 vhdx are.

**Windows Storage Sense** — built in, automated, shallow. It clears temp files, the recycle bin, and old update files on a schedule; independent comparisons routinely find a manual pass with a dedicated tool still recovers meaningfully more space, because Storage Sense has no idea what a stale build artifact is either.

## 3. Positioning Statement

Not a tweak dashboard. Not a registry cleaner. Not a game. A precision instrument for developers and power users whose drives are quietly full of dead build output, that finds it in seconds using the same trick WizTree uses, tells you exactly what's safe to remove and why, and gets out of the way. The bar is chippytea's actual product quality, not its marketing skin.

## 4. Explicit Anti-Scope — Do Not Build These in v1, and Think Hard Before Ever Building Them

- No gamified currency, counters, mascots, or songs. The chip economy is the one part of the source material to leave behind entirely.
- No registry cleaner. This is the single feature most responsible for CCleaner's reputation for breaking things.
- No "AI-powered system health score." Cosmetic scoring like this has already been called out publicly as marketing theater layered onto CCleaner — don't repeat it.
- No driver updater, no bundled toolbar/offer of any kind, ever.
- No app-installer/winget wrapper, no system-tweaks panel — that's WinUtil's job, not this app's.
- No forced telemetry, no subscription nag screens, no dark-pattern upsell.
- Nothing is deleted without an explicit review step. Ever.

## 5. Core v1 Feature Set

**Discovery (the MFT-speed part).** Enumerate the volume via direct Master File Table / USN journal reads — not a recursive directory walk — the same technique that makes WizTree fast. Rust crates `ntfs-reader` or `usn-journal-rs` do this today. The dev-artifact classification below runs as a filter pass over the resulting flat index, not a second slow scan.

**Classification (the judgment part).** Recognize, by manifest/lockfile presence, the categories actually worth reclaiming on a dev machine:
- `node_modules` next to a `package.json`
- Rust `target/` next to a `Cargo.toml`
- Python `venv`/`.venv`/`__pycache__` next to `requirements.txt` or `pyproject.toml`
- `.gradle`, Maven `.m2`, NuGet package cache
- Visual Studio `obj/`/`bin/`/`.vs`, JetBrains IDE caches
- Unity `Library/`/`Temp/`
- Docker image/layer bloat, WSL2 distro `.vhdx` files that have grown but never compacted
- Stale installers sitting in Downloads (`.exe`/`.msi` older than N days)
- Windows Update leftovers and `Windows.old` after upgrades

**Safety checks before anything is ever offered as removable** — this is the actual product, more than the scan speed:
- Skip anything inside a Git repo with uncommitted changes
- Skip OneDrive/cloud placeholder files (reparse-point "online-only" attributes) so nothing triggers an unwanted redownload or gets deleted out from under a sync
- Skip anything with an open file handle / in active use
- Respect junctions and symlinks instead of following them into surprising places
- Show size, last-modified date, and a one-line reason for every suggestion

**Review-before-delete flow:** Recycle Bin by default; permanent delete is an explicit opt-in per category, never the default. A plain history log of what was removed and how much space came back — no counters, no currency, no sound effects.

## 6. Design and Brand Direction

- Native Fluent 2: Mica material, system accent color, automatic light/dark, Segoe UI Variable. Not a webview wearing a Windows skin.
- No mascot, no illustrated theme. chippytea's fish-and-chips branding is a fun regional pun for one UK developer's side project; it doesn't transplant, and a straight visual copy would look derivative. This needs its own identity.
- Naming: avoid "Cleaner / Booster / Optimizer / Turbo" — that category is exactly the one CCleaner and the IObit-style tools have poisoned. Picked: **Undrift**.
- Motion should be functional: a scan reveals results progressively because the MFT read is fast enough to show live, not because it's animated for effect. No confetti on a successful clean.

## 7. Technical Architecture

Same division of labor chippytea uses — native UI in front, Rust doing the real work underneath — translated to Windows equivalents rather than copied as SwiftUI:

- **Core engine: Rust**, compiled as a library (`cdylib`). Owns MFT/USN scanning, classification rules, safety checks, and the cleanup executor. This is where `ntfs-reader`/`usn-journal-rs` live, plus `git2` for the dirty-repo check.
- **Shell: C#/.NET 8 + WinUI 3**, not windows-rs. Getting Fluent 2/Mica for free from the platform by writing the shell in the language WinUI 3 actually targets is the pragmatic call; the Rust core still does 100% of the scanning and deletion logic. Call across the boundary with a thin FFI (JSON-over-stdio for CLI and direct P/Invoke bindings).
- **No resident background service in v1.** Launch, scan, act, quit. A lightweight scheduled-task-based reminder is a plausible v1.1 idea, not a requirement — and if it ships, it stays silent by default and sends nothing over the network either way.
- Elevation is requested per-operation where actually required (some dev caches live under paths a standard user can't touch), never as an "always run as admin" blanket requirement.

## 8. Distribution and Trust

- Standard OV certificate or Azure Artifact Signing.
- Two channels:
  1. Signed installer + winget manifest (`winget install Undrift`).
  2. MSIX listing on the Microsoft Store.

## 9. Performance Target Numbers

- Cold start under 1 second
- Idle memory under 50MB
- Full-drive initial scan in single-digit seconds on a modern NVMe drive (MFT read, not a directory walk)
- Zero network calls except an explicit update check; zero telemetry, period
