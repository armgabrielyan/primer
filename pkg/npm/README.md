# primer npm package

This is the npm distribution package for [primer](https://github.com/armgabrielyan/primer), a CLI for AI-guided project recipes and milestone workflows.

## Installation

```bash
npm install -g primer-cli
```

Or run directly with npx:

```bash
npx primer-cli list
```

## What this package does

When you install this package, it automatically downloads the appropriate pre-built binary for your platform from GitHub Releases. Supported platforms:

- macOS (Intel and Apple Silicon)
- Linux (x64 and ARM64)
- Windows (x64)

## Alternative installation methods

### Cargo (from source)

```bash
cargo install primer
```

### Homebrew (macOS/Linux)

```bash
brew install armgabrielyan/tap/primer
```

## Usage

```bash
primer list
primer init operating-system --tool codex --path ../my-os
primer status
```

For more information, see the [full documentation](https://github.com/armgabrielyan/primer).

## License

MIT
