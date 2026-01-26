# Windows Build Prerequisites

This document covers the prerequisites needed to build Jana on Windows.

## Required Software

### 1. Node.js (v18+)

Download and install from [nodejs.org](https://nodejs.org/) or via winget:

```powershell
winget install OpenJS.NodeJS.LTS
```

Verify installation:
```powershell
node --version
npm --version
```

### 2. Rust Toolchain

Install via rustup:

```powershell
winget install Rustlang.Rustup
```

Or download the installer from [rustup.rs](https://rustup.rs/).

After installation, **restart your terminal** and verify:

```powershell
rustc --version
cargo --version
```

### 3. Visual Studio Build Tools

Rust on Windows requires the MSVC C++ build tools. Install Visual Studio Build Tools with the "Desktop development with C++" workload:

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

This installs:
- MSVC v143 compiler
- Windows 11 SDK
- C++ CMake tools
- Other required components

### 4. WebView2 Runtime

Tauri uses WebView2 for rendering. It's pre-installed on Windows 11, but on Windows 10 you may need to install it:

```powershell
winget install Microsoft.EdgeWebView2Runtime
```

## Build Instructions

1. Clone the repository:
   ```bash
   git clone https://github.com/peak-flow/jana-ai.git
   cd jana-ai
   ```

2. Install npm dependencies:
   ```bash
   npm install
   ```

3. Build the application:
   ```bash
   npm run tauri build
   ```

4. Find the installers at:
   - `src-tauri/target/release/bundle/msi/Jana_*.msi`
   - `src-tauri/target/release/bundle/nsis/Jana_*-setup.exe`

## Troubleshooting

### Icon Error: "old DIB in icon.ico"

If you see an error like:
```
error RC2176 : old DIB in icon.ico; pass it through SDKPAINT
```

The `icon.ico` file is malformed. Regenerate it from the PNG sources:

```bash
npm install --save-dev png-to-ico
node -e "
const pngToIco = require('png-to-ico').default;
const fs = require('fs');
pngToIco(['src-tauri/icons/32x32.png', 'src-tauri/icons/128x128.png', 'src-tauri/icons/128x128@2x.png'])
  .then(buf => fs.writeFileSync('src-tauri/icons/icon.ico', buf));
"
```

### Rust Not Found After Installation

If `rustc` isn't recognized after installing rustup, ensure `%USERPROFILE%\.cargo\bin` is in your PATH. You may need to:
- Restart your terminal
- Log out and back in
- Or manually add to PATH

### sqlx Compile-Time Checking

If you encounter sqlx compile errors related to database queries, set the offline mode environment variable:

```powershell
$env:SQLX_OFFLINE = "true"
npm run tauri build
```

Note: The current codebase uses runtime queries, so this shouldn't be necessary.

### Visual Studio Build Tools Not Detected

Ensure you installed the "Desktop development with C++" workload, not just the base Build Tools. You can modify the installation:

1. Open "Visual Studio Installer"
2. Click "Modify" on Build Tools 2022
3. Check "Desktop development with C++"
4. Click "Modify" to install

## Version Requirements

| Tool | Minimum Version | Tested With |
|------|-----------------|-------------|
| Node.js | 18.x | 25.2.1 |
| Rust | 1.70+ | 1.93.0 |
| VS Build Tools | 2019+ | 2022 |
