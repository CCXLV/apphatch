# AppHatch

A simple AppImage management tool for Linux that helps you install AppImage applications across different Linux distributions.

## Features

- Install AppImage applications from various sources
- Manage AppImage configurations and metadata
- Cross-distribution compatibility
- Simple command-line interface
- Automatic AppImage integration

## Installation

Download the latest release for your distribution:

- **Debian/Ubuntu**: `.deb` package
- **Fedora**: `.rpm` package  
- **Arch Linux**: `.pkg.tar.zst` package

## Usage

```bash
# Show help
apphatch --help

# Install an AppImage
apphatch install -p <appimage-file-path>

# Uninstall an AppImage
apphatch uninstall -n <app-name>
```

## Building from Source

### Prerequisites

- Rust 1.70+ 
- Cargo

### Build

```bash
git clone https://github.com/ccxlv/apphatch.git
cd apphatch
cargo build --release
```

The binary will be available at `target/release/apphatch`.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

