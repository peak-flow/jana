# Build Guide

Instructions for building Jana on macOS (Intel & Apple Silicon), Windows, and Linux.

## Quick Start

```bash
# Install dependencies
npm install

# Development mode (hot reload)
npm run tauri dev

# Production build (native platform)
npm run tauri build
```

---

## macOS (Apple Silicon / M1, M2, M3, M4)

### Prerequisites

1. **Xcode Command Line Tools**
   ```bash
   xcode-select --install
   ```

2. **Node.js** (LTS recommended)
   ```bash
   # Using Homebrew
   brew install node
   ```

3. **Rust**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

### Build

```bash
npm install
npm run tauri build
```

**Output:** `src-tauri/target/release/bundle/dmg/Jana_0.1.0_aarch64.dmg`

### Cross-compile for Intel Macs (from Apple Silicon)

If you want to build an Intel version from your M-series Mac:

```bash
# One-time setup: add Intel target
rustup target add x86_64-apple-darwin

# Build for Intel
npm run tauri build -- --target x86_64-apple-darwin
```

**Output:** `src-tauri/target/x86_64-apple-darwin/release/bundle/dmg/Jana_0.1.0_x64.dmg`

---

## macOS (Intel)

### Prerequisites

Same as Apple Silicon:
1. Xcode Command Line Tools
2. Node.js
3. Rust

### Build

```bash
npm install
npm run tauri build
```

**Output:** `src-tauri/target/release/bundle/dmg/Jana_0.1.0_x64.dmg`

---

## Windows

### Prerequisites

1. **Microsoft Visual Studio C++ Build Tools**
   - Download from [Visual Studio Downloads](https://visualstudio.microsoft.com/downloads/)
   - Select "Desktop development with C++" workload
   - Or install via winget:
     ```powershell
     winget install Microsoft.VisualStudio.2022.BuildTools
     ```

2. **WebView2**
   - Usually pre-installed on Windows 10/11
   - If missing: [Download WebView2](https://developer.microsoft.com/microsoft-edge/webview2/)

3. **Node.js**
   ```powershell
   winget install OpenJS.NodeJS.LTS
   ```

4. **Rust**
   - Download and run [rustup-init.exe](https://rustup.rs/)
   - Or via winget:
     ```powershell
     winget install Rustlang.Rustup
     ```

### Build

```powershell
npm install
npm run tauri build
```

**Output:** `src-tauri\target\release\bundle\nsis\Jana_0.1.0_x64-setup.exe`

### Building Windows from macOS

Tauri cannot cross-compile to Windows from macOS. Options:

1. **GitHub Actions** — Add a workflow that builds on `windows-latest` runner
2. **Parallels/VMware** — Run Windows in a VM and build natively
3. **Real Windows machine** — Build directly on Windows hardware

For most users, shipping x64 Windows only is sufficient — ARM Windows can run x64 apps via emulation.

---

## Linux

### Prerequisites (Ubuntu/Debian)

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

Install Node.js:
```bash
curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -
sudo apt install -y nodejs
```

Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Build

```bash
npm install
npm run tauri build
```

**Output:**
- `src-tauri/target/release/bundle/deb/jana_0.1.0_amd64.deb`
- `src-tauri/target/release/bundle/appimage/jana_0.1.0_amd64.AppImage`

---

## Output Summary

All build artifacts are in `src-tauri/target/`:

| Platform | Architecture | Output Path |
|----------|--------------|-------------|
| macOS | Apple Silicon | `target/release/bundle/dmg/Jana_0.1.0_aarch64.dmg` |
| macOS | Intel | `target/release/bundle/dmg/Jana_0.1.0_x64.dmg` |
| macOS | Intel (cross) | `target/x86_64-apple-darwin/release/bundle/dmg/Jana_0.1.0_x64.dmg` |
| Windows | x64 | `target\release\bundle\nsis\Jana_0.1.0_x64-setup.exe` |
| Linux | x64 | `target/release/bundle/deb/jana_0.1.0_amd64.deb` |

Cross-compile targets use their own subdirectory and never overwrite native builds.

---

## Troubleshooting

### Rust not found after install
```bash
source ~/.cargo/env
# Or restart your terminal
```

### macOS: "Developer cannot be verified"
See [README.md](../README.md#installation-unsigned-app) for Gatekeeper bypass instructions.

### Windows: WebView2 missing
Download and install from [Microsoft](https://developer.microsoft.com/microsoft-edge/webview2/).

### Linux: Missing dependencies
The build will fail with clear error messages about missing libraries. Install the listed packages and retry.
