# DENGINKS OPC-UA Diagnostic Tool

A lightweight, portable, read-only OPC-UA Client for diagnostics, structural exporting, and data monitoring.

![Aesthetic Dashboard](https://raw.githubusercontent.com/DigitalEnginks/opcua-diagnostic-tool/main/assets/screenshot.png)

## Features

- **Read-Only by Design**: Safe for production environments. No write or method call capabilities.
- **Network Pre-check**: Verify TCP connectivity and latency before connecting.
- **Endpoint Discovery**: Automatically find and list available security policies and authentication methods.
- **Structural Browsing**: Intuitive tree view of the OPC-UA address space with lazy loading.
- **Real-time Monitoring**: Watch multiple variables simultaneously with live value updates and quality status.
- **Trending**: Visualize numeric data in real-time charts.
- **Network Crawler**: Recursively discover nodes and export the structure.
- **Data Export**: Export monitored data and crawler results to CSV and JSON formats.
- **Multi-language**: Full support for English and Spanish.
- **Portable**: Single executable with file-based bookmark management.

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- Visual Studio Build Tools (Windows) or standard C development tools (Linux/macOS)

### Build from source

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

For more internal details, see:
- [Architecture Guide](docs/architecture.md)
- [i18n System](docs/i18n.md)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Disclaimer

This tool is provided "as is" without warranty of any kind. Always verify tool behavior in a safe environment before using on critical production systems.
