# WINDOWS_SETUP.md — Developing Sweepie on Windows

> Guide for setting up your Windows developer environment to build, run, and test Sweepie.

## Prerequisites

1. **Rust Toolchain (MSVC ABI)**:
   - Install via [rustup.rs](https://rustup.rs/) (choose `x86_64-pc-windows-msvc`).
   ```powershell
   rustup default stable-x86_64-pc-windows-msvc
   rustup component add clippy rustfmt
   ```

2. **Visual Studio 2022**:
   - Community, Professional, or Enterprise.
   - Workloads required:
     - **.NET Desktop Development** (includes .NET 8 SDK)
     - **Windows application development** (includes Windows App SDK C# templates and Windows 10/11 SDKs)
     - **Desktop development with C++** (provides MSVC linker and headers for Rust crates)

3. **.NET 8 SDK**:
   - Verify installation:
   ```powershell
   dotnet --version
   # Expected: 8.0.xxx
   ```

---

## Step-by-Step Setup

### 1. Clone the Repository

```powershell
git clone https://github.com/diluteoxygen/sweepie.git
cd sweepie
```

### 2. Build the Core Rust Engine

From the repository root:

```powershell
# Run the test suite
cargo test

# Verify zero clippy warnings
cargo clippy --all-targets -- -D warnings

# Build release binaries (produces target/release/sweepie.exe and sweepie_core.dll)
cargo build --release
```

### 3. Test the Core CLI on Windows

Run a live MFT scan on your `C:` volume (requires administrative prompt for raw disk handle access):

```powershell
# Run human-readable scan
.\target\release\sweepie.exe scan C: --all

# Run scan with JSON output
.\target\release\sweepie.exe scan C: --json
```

### 4. Build and Run the WinUI 3 App

You can build and run via command line or Visual Studio:

**Option A: Command Line (`dotnet`)**:
```powershell
dotnet restore app/Sweepie.App.sln
dotnet build app/Sweepie.App/Sweepie.App.csproj --configuration Release
dotnet run --project app/Sweepie.App/Sweepie.App.csproj --configuration Release
```

**Option B: Visual Studio 2022**:
1. Double click `app/Sweepie.App.sln` to open the solution.
2. Select target platform `x64` and configuration `Release` (or `Debug`).
3. Set `Sweepie.App` as the Startup Project.
4. Press `F5` to launch with debugging.

---

## Troubleshooting

- **Elevation error during MFT scan**:
  Reading the raw NTFS volume handle (`\\.\C:`) requires Administrator privileges. Run Windows Terminal or PowerShell as Administrator when testing MFT enumeration, or let the CLI fall back to directory traversal.
- **Missing DLL error on launch**:
  The `.csproj` automatically copies `sweepie.exe` and `sweepie_core.dll` from `target/release/` into the application build output. Ensure you ran `cargo build --release` before running the WinUI 3 app.
