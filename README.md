# Gosh USB Creator

A cross-platform USB flash drive creator for Windows, Linux, and macOS. Built with Tauri v2 (Rust) and React/TypeScript.

## Features

- Write ISO/IMG files to USB drives
- SHA-256 and MD5 checksum verification
- Optional write verification (read-back compare)
- Cross-platform: Windows, Linux, macOS
- Light/Dark/System theme support
- No telemetry, accounts, or cloud features

## Requirements

### All Platforms
- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) 1.70+

### Linux
- `libwebkit2gtk-4.1-dev`
- `libappindicator3-dev`
- `librsvg2-dev`

On Debian/Ubuntu:
```bash
sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev
```

### Windows
- WebView2 (included in Windows 10/11)

### macOS
- Xcode Command Line Tools

## Building

```bash
# Install dependencies
npm install

# Development
npm run tauri dev

# Production build
npm run tauri build
```

## Usage

1. **Select ISO** - Click or drag an ISO/IMG file
2. **Verify checksum** (optional) - Calculate SHA-256 or MD5 and compare
3. **Select USB device** - Choose the target drive
4. **Write** - Click "Write to USB" (requires admin privileges)

## Permissions

Writing to raw block devices requires elevated privileges:

- **Linux**: Uses `pkexec` for privilege elevation
- **macOS**: Prompts for admin password
- **Windows**: Requires running as Administrator

## License

AGPL-3.0 - See [LICENSE](LICENSE)

This license requires:
- Source code must be provided when distributing modified versions
- Network use (SaaS) must provide source access
- No closed-source forks

## Contributing

Contributions welcome. Please open an issue first for major changes.
