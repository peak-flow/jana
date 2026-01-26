# Build Guide

## Prerequisites

- Node.js (LTS)
- Rust (via rustup)
- Platform-specific dependencies (see below)

## Development

```bash
npm run tauri dev
```

## Production Builds

### macOS ARM (Apple Silicon) — Native

```bash
npm run tauri build
```

Output: `src-tauri/target/release/bundle/dmg/Jana_0.1.0_aarch64.dmg`

### macOS Intel (x86_64) — Cross-compile from ARM

One-time setup:

```bash
rustup target add x86_64-apple-darwin
```

Build:

```bash
npm run tauri build -- --target x86_64-apple-darwin
```

Output: `src-tauri/target/x86_64-apple-darwin/release/bundle/dmg/Jana_0.1.0_x64.dmg`

To remove the Intel target after building:

```bash
rustup target remove x86_64-apple-darwin
```

Adding/removing targets has no performance impact — it only downloads/deletes ~30MB of static library files.

### Windows

Tauri cannot cross-compile to Windows from macOS. Options:

1. **GitHub Actions** — easiest. Use a Windows runner in CI to build automatically.
2. **Parallels (ARM Mac)** — runs ARM Windows, builds `aarch64-pc-windows-msvc` natively. Can cross-compile to x86_64 with `rustup target add x86_64-pc-windows-msvc`.
3. **Native Windows machine** — install Rust, Node.js, and MSVC build tools, then run `npm run tauri build`.

For distribution, shipping x86_64 Windows only is typically sufficient — ARM Windows runs x86_64 apps via emulation.

### Linux

From a Linux machine (or CI):

```bash
npm run tauri build
```

Produces `.deb` and `.AppImage` bundles.

## Output Locations

All bundles are under `src-tauri/target/`:

| Target | Path |
|--------|------|
| Native (ARM mac) | `target/release/bundle/` |
| Intel mac | `target/x86_64-apple-darwin/release/bundle/` |
| Windows | `target/release/bundle/` (on Windows) |
| Linux | `target/release/bundle/` (on Linux) |

Different cross-compile targets get their own subdirectory and never overwrite each other.
