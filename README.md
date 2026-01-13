# DENGINKS OPC-UA Diagnostic Tool

[![Build Status](https://img.shields.io/github/actions/workflow/status/DigitalEnginks/opcua-diagnostic-tool/rust.yml?style=for-the-badge)](https://github.com/DigitalEnginks/opcua-diagnostic-tool/actions)
[![License](https://img.shields.io/github/license/DigitalEnginks/opcua-diagnostic-tool?style=for-the-badge)](LICENSE)
[![Version](https://img.shields.io/github/v/release/DigitalEnginks/opcua-diagnostic-tool?style=for-the-badge)](https://github.com/DigitalEnginks/opcua-diagnostic-tool/releases)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org)

Welcome to **DENGINKS OPC-UA Diagnostic Tool**! 🚀

A lightweight, portable, **read-only** OPC-UA Client designed for diagnostics, structural exporting, and data monitoring without the risk of accidentally modifying server data.

![Aesthetic Dashboard](https://raw.githubusercontent.com/DigitalEnginks/opcua-diagnostic-tool/main/assets/screenshot.png)

## Features

- 🛡️ **Read-Only by Design**: Safe for production environments. No write or method call capabilities.
- 📡 **Network Pre-check**: Verify TCP connectivity and latency before connecting.
- 🔍 **Endpoint Discovery**: Automatically find and list available security policies and authentication methods.
- 🌳 **Structural Browsing**: Intuitive tree view of the OPC-UA address space with lazy loading.
- 📈 **Real-time Monitoring**: Watch multiple variables simultaneously with live value updates and quality status.
- 📊 **Trending**: Visualize numeric data in real-time charts.
- 🕷️ **Network Crawler**: Recursively discover nodes and export the structure.
- 💾 **Data Export**: Export monitored data and crawler results to CSV and JSON formats.
- 🌍 **Multi-language**: Full support for English and Spanish.
- 🎒 **Portable**: Single executable with file-based bookmark management.

## Installation & Compatibility

### Windows (Primary Support) 🪟

This tool is **developed and optimized for Windows**. It is provided as a standalone executable that works out-of-the-box on most modern Windows systems.

**Prerequisites for building:**
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- Visual Studio Build Tools (C++ workload)

**Compatibility Note:**
The build script automatically includes Mesa3D `opengl32.dll` to support software rendering on VMs or systems without modern GPUs.

### Linux (Experimental) 🐧

While Rust is cross-platform, this application uses hardware-accelerated GUI libraries that require specific system dependencies on Linux. We do not provide official pre-built binaries for Linux, but you can compile it yourself.

**You must install these dependencies before building:**

```bash
# Ubuntu / Debian
sudo apt-get update
sudo apt-get install -y libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libx11-dev libasound2-dev libudev-dev
```

## Build & Distribution

The easiest way to get a ready-to-use **Portable ZIP** is by using our included PowerShell script.

### Create Portable Package (Recommended)

Run the following command in PowerShell:

```powershell
.\build.ps1 -Release -Package
```

This will:
1.  Compile the application in release mode.
2.  Create a `dist/` folder.
3.  Include the executable and the necessary compatibility drivers (`opengl32.dll`).
4.  Generate a `denginks-opcua-diagnostic-portable.zip` ready to share.

### Manual Build

```powershell
git clone https://github.com/DigitalEnginks/opcua-diagnostic-tool.git
cd opcua-diagnostic-tool
cargo build --release
```

The executable will be located in `target/release/denginks-opcua-diagnostic.exe`.

## Usage

1.  **Connection**: Enter the server endpoint URL (e.g., `opc.tcp://localhost:4840`).
2.  **Check**: Click "Check Network" to verify connectivity or "Discover Endpoints" to see security options.
3.  **Connect**: Select an endpoint and click "Connect".
4.  **Explore**: Use the Tree View on the left to browse the address space.
5.  **Monitor**: Right-click or use the properties panel to add variables to the Watchlist.
6.  **Analyze**: Use the Trending tab to see data changes over time.

## Documentation

For more internal details for developers, check our docs:
- [Architecture Guide](docs/architecture.md)
- [i18n System](docs/i18n.md)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Disclaimer

This software is provided **"as is"**, without warranty of any kind, express or implied.

*   **Windows Users:** The tool is tested primarily on Windows 10/11.
*   **Linux/macOS Users:** Support is experimental. Functionality "as-is" implies no guarantee of UI consistency or hardware acceleration support on non-Windows platforms.

Always verify tool behavior in a non-critical environment before using it on production systems.

---

**Made in Chile 🇨🇱**
*Why? Because Chile is the best country in Chile.*
