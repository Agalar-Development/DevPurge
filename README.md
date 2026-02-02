# DevPurge 🧹

A powerful CLI tool to clean up build artifacts and dependency folders from your development projects, helping you reclaim valuable disk space.

## Features ✨

- 🔍 **Smart Scanning**: Automatically detects dependency and build folders across multiple project types
- 🛡️ **Safety First**: Validates folders before deletion by checking for project configuration files
- 📊 **Interactive Selection**: Multi-select interface to choose which folders to delete
- 💾 **Cache Support**: Speed up repeated scans with intelligent caching
- 📈 **Progress Tracking**: Visual progress bars for scanning and deletion operations
- 🎯 **Size Filtering**: Filter folders by minimum size to focus on the biggest space hogs
- 🌈 **Multi-Language Support**: Works with JavaScript/TypeScript, Rust, Java, Python, .NET, Dart, PHP, Go, and more

## Supported Folder Types

DevPurge can safely identify and remove the following dependency/build folders:

| Folder | Project Type | Verification File |
|--------|--------------|-------------------|
| `node_modules` | JavaScript/TypeScript | package.json |
| `target` | Rust | Cargo.toml |
| `build` | Java/Gradle/C++/Angular | pom.xml, build.gradle, CMakeLists.txt, angular.json |
| `dist` | Web Projects | package.json, angular.json, vite.config.js |
| `.gradle` | Gradle | build.gradle, settings.gradle |
| `vendor` | PHP/Go/Ruby | composer.json, go.mod, Gemfile |
| `__pycache__` | Python | (always safe) |
| `bin`, `obj` | .NET | .csproj, .fsproj, .sln |
| `.dart_tool` | Dart | pubspec.yaml |
| `.angular` | Angular | angular.json |
| `.next` | Next.js | next.config.js |
| `.nuxt` | Nuxt.js | nuxt.config.js |

## Installation

### From Source

```bash
git clone https://github.com/agalar-development/DevPurge.git
cd DevPurge
cargo build --release
```

The binary will be available at `target/release/devpurge.exe` (Windows) or `target/release/devpurge` (Linux/macOS).

## Usage

### Basic Usage

Scan the current directory:
```bash
devpurge
```

Scan a specific directory:
```bash
devpurge --path "C:\Users\YourName\Projects"
```

### Command-Line Options

```bash
Options:
  -p, --path <PATH>          Path to scan for dependency folders
  -m, --min-size <MIN_SIZE>  Minimum folder size in MB (default: 0)
      --scan                 Force a new scan (ignore cache)
      --no-cache             Don't use or save cache
  -h, --help                 Print help
  -V, --version              Print version
```

### Examples

Scan and find all folders:
```bash
devpurge --path ~/Projects
```

Only show folders larger than 500 MB:
```bash
devpurge --path ~/Projects --min-size 500
```

Force a fresh scan without using cache:
```bash
devpurge --scan
```

## How It Works

1. **Scanning**: DevPurge walks through your directory tree looking for common dependency and build folders
2. **Validation**: Before marking a folder for deletion, it checks for the presence of project configuration files to ensure it's safe to delete
3. **Selection**: You can interactively select which folders to delete using arrow keys and spacebar
4. **Deletion**: After confirmation, selected folders are permanently removed
5. **Caching**: Scan results are cached to speed up future runs (cache is automatically updated after deletion)

## Safety Features

- **Project File Verification**: Each folder type is validated against its corresponding project configuration file
- **Explicit Confirmation**: Requires typing "yes" to confirm deletion
- **Clear Reporting**: Shows which folders will be deleted and how much space will be reclaimed
- **Cache Updates**: Automatically removes deleted folders from cache to prevent stale results

## Cache Location

DevPurge stores its cache at:
- **Windows**: `C:\Users\<username>\AppData\Local\devpurge\devpurge\cache\scan_cache.json`
- **Linux**: `~/.cache/devpurge/scan_cache.json`
- **macOS**: `~/Library/Caches/devpurge/scan_cache.json`

## Example Output

```
DevPurge - Developer Dependency Cleaner
Scanning C:\Users\mert\Projects for dependency folders... This may take a while.
Found 15 folders. Total size: 8.5 GB

Select folders to DELETE (Up/Down to move, Space to toggle, Enter to confirm)
[x] C:\...\project1\node_modules (2.3 GB)
[x] C:\...\project2\target (1.8 GB)
[x] C:\...\project3\build (900 MB)
...

Cleanup complete! Reclaimed space: 8.5 GB
```

## Building

This project requires Rust 1.70 or later.

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with cargo
cargo run -- --help
```

## Dependencies

- `walkdir` - Directory traversal
- `dialoguer` - Interactive CLI prompts
- `indicatif` - Progress bars and spinners
- `human_bytes` - Human-readable byte formatting
- `clap` - Command-line argument parsing
- `serde` & `serde_json` - Cache serialization
- `directories` - Platform-specific directory paths
- `anyhow` - Error handling
- `console` - Terminal manipulation

## Contributing

Contributions are welcome! Feel free to:
- Report bugs
- Suggest new features
- Add support for more project types
- Improve documentation

## License

This project is open source and available under the MIT License.

## Warning ⚠️

This tool permanently deletes files. Always ensure you have backups and can regenerate the deleted folders (e.g., via `npm install`, `cargo build`, etc.) before using DevPurge.

## Author

Created by Luxotick

---

**Happy Cleaning! 🧹✨**
