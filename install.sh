#!/bin/sh
# primer installer script
# Usage: curl -sSf https://raw.githubusercontent.com/armgabrielyan/primer/main/install.sh | sh
#
# This script detects your OS and architecture, downloads the appropriate
# pre-built binary, and installs it to ~/.local/bin (or /usr/local/bin with sudo).

set -e

REPO="armgabrielyan/primer"
BINARY_NAME="primer"
INSTALL_DIR="${PRIMER_INSTALL_DIR:-$HOME/.local/bin}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info() {
    printf "${BLUE}info:${NC} %s\n" "$1"
}

success() {
    printf "${GREEN}success:${NC} %s\n" "$1"
}

warn() {
    printf "${YELLOW}warning:${NC} %s\n" "$1"
}

error() {
    printf "${RED}error:${NC} %s\n" "$1" >&2
    exit 1
}

detect_os() {
    case "$(uname -s)" in
        Linux*) echo "linux" ;;
        Darwin*) echo "macos" ;;
        MINGW*|MSYS*|CYGWIN*) echo "windows" ;;
        *) error "Unsupported operating system: $(uname -s)" ;;
    esac
}

detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64) echo "x86_64" ;;
        aarch64|arm64) echo "aarch64" ;;
        *) error "Unsupported architecture: $(uname -m)" ;;
    esac
}

get_target() {
    os="$1"
    arch="$2"

    case "$os" in
        linux)
            if ldd --version 2>&1 | grep -q musl; then
                echo "${arch}-unknown-linux-musl"
            else
                echo "${arch}-unknown-linux-gnu"
            fi
            ;;
        macos)
            echo "${arch}-apple-darwin"
            ;;
        windows)
            echo "${arch}-pc-windows-msvc"
            ;;
    esac
}

get_latest_version() {
    version=$(curl -sSf "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"v([^"]+)".*/\1/')

    if [ -z "$version" ]; then
        error "Failed to fetch latest version from GitHub"
    fi

    echo "$version"
}

check_requirements() {
    if ! command -v curl >/dev/null 2>&1; then
        error "curl is required but not installed"
    fi

    if ! command -v tar >/dev/null 2>&1; then
        error "tar is required but not installed"
    fi
}

install_binary() {
    os=$(detect_os)
    arch=$(detect_arch)
    target=$(get_target "$os" "$arch")

    info "Detected platform: $os ($arch)"
    info "Target: $target"

    info "Fetching latest version..."
    version=$(get_latest_version)
    info "Latest version: v$version"

    if [ "$os" = "windows" ]; then
        archive_ext="zip"
    else
        archive_ext="tar.gz"
    fi

    archive_name="primer-${version}-${target}.${archive_ext}"
    download_url="https://github.com/${REPO}/releases/download/v${version}/${archive_name}"

    info "Downloading $archive_name..."

    tmp_dir=$(mktemp -d)
    trap 'rm -rf "$tmp_dir"' EXIT

    if ! curl -sSfL "$download_url" -o "$tmp_dir/$archive_name"; then
        error "Failed to download from $download_url"
    fi

    info "Extracting..."
    cd "$tmp_dir"

    if [ "$archive_ext" = "zip" ]; then
        unzip -q "$archive_name"
    else
        tar -xzf "$archive_name"
    fi

    if [ ! -d "$INSTALL_DIR" ]; then
        info "Creating directory $INSTALL_DIR"
        mkdir -p "$INSTALL_DIR"
    fi

    info "Installing to $INSTALL_DIR/$BINARY_NAME"

    if [ -w "$INSTALL_DIR" ]; then
        cp "$BINARY_NAME" "$INSTALL_DIR/"
        chmod +x "$INSTALL_DIR/$BINARY_NAME"
    else
        warn "Cannot write to $INSTALL_DIR, trying with sudo..."
        sudo cp "$BINARY_NAME" "$INSTALL_DIR/"
        sudo chmod +x "$INSTALL_DIR/$BINARY_NAME"
    fi

    success "primer v$version installed successfully!"

    case ":$PATH:" in
        *":$INSTALL_DIR:"*)
            info "Run 'primer --help' to get started"
            ;;
        *)
            warn "$INSTALL_DIR is not in your PATH"
            echo ""
            echo "Add it to your PATH by adding this line to your shell config:"
            echo ""
            echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
            echo ""
            ;;
    esac
}

main() {
    echo ""
    echo "  ╭──────────────────────────────────────────────╮"
    echo "  │  primer installer                            │"
    echo "  │  AI-guided project recipes and milestones    │"
    echo "  ╰──────────────────────────────────────────────╯"
    echo ""

    check_requirements
    install_binary
}

main "$@"
