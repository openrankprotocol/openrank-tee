#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO="openrankprotocol/openrank-tee"
BINARY_NAME="openrank"
INSTALL_DIR="/usr/local/bin"

# Print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Detect OS and architecture
detect_platform() {
    local os=""
    local arch=""

    case "$(uname -s)" in
        Linux*)     os="linux" ;;
        Darwin*)    os="macos" ;;
        CYGWIN*|MINGW*|MSYS*) os="windows" ;;
        *)
            print_error "Unsupported operating system: $(uname -s)"
            exit 1
            ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64)   arch="amd64" ;;
        aarch64|arm64)  arch="arm64" ;;
        *)
            print_error "Unsupported architecture: $(uname -m)"
            exit 1
            ;;
    esac

    if [ "$os" = "windows" ]; then
        echo "${BINARY_NAME}-${os}-${arch}.exe"
    else
        echo "${BINARY_NAME}-${os}-${arch}"
    fi
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Download file with progress
download_file() {
    local url="$1"
    local output="$2"

    if command_exists curl; then
        curl -fsSL --progress-bar "$url" -o "$output"
    elif command_exists wget; then
        wget --progress=bar:force -O "$output" "$url"
    else
        print_error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi
}

# Get latest release version
get_latest_version() {
    local api_url="https://api.github.com/repos/${REPO}/releases/latest"

    if command_exists curl; then
        curl -fsSL "$api_url" | grep '"tag_name"' | cut -d'"' -f4
    elif command_exists wget; then
        wget -qO- "$api_url" | grep '"tag_name"' | cut -d'"' -f4
    else
        print_error "Neither curl nor wget found. Cannot fetch latest version."
        exit 1
    fi
}

# Verify checksum
verify_checksum() {
    local file="$1"
    local checksums_file="$2"

    if [ ! -f "$checksums_file" ]; then
        print_warning "Checksums file not found. Skipping verification."
        return 0
    fi

    if command_exists sha256sum; then
        local expected_hash=$(grep "$(basename "$file")" "$checksums_file" | cut -d' ' -f1)
        local actual_hash=$(sha256sum "$file" | cut -d' ' -f1)

        if [ "$expected_hash" = "$actual_hash" ]; then
            print_success "Checksum verification passed"
            return 0
        else
            print_error "Checksum verification failed"
            print_error "Expected: $expected_hash"
            print_error "Actual:   $actual_hash"
            return 1
        fi
    else
        print_warning "sha256sum not found. Skipping checksum verification."
        return 0
    fi
}

# Main installation function
install_openrank() {
    local version="$1"
    local force_install="$2"

    print_status "Starting OpenRank CLI installation..."

    # Detect platform
    local asset_name
    asset_name=$(detect_platform)
    print_status "Detected platform: $asset_name"

    # Get version
    if [ -z "$version" ]; then
        print_status "Fetching latest version..."
        version=$(get_latest_version)
        if [ -z "$version" ]; then
            print_error "Failed to fetch latest version"
            exit 1
        fi
    fi

    print_status "Installing version: $version"

    # Check if already installed
    if command_exists "$BINARY_NAME" && [ "$force_install" != "true" ]; then
        local current_version
        current_version=$($BINARY_NAME --version 2>/dev/null | grep -oE 'v[0-9]+\.[0-9]+\.[0-9]+' || echo "unknown")

        if [ "$current_version" = "$version" ]; then
            print_success "OpenRank CLI $version is already installed"
            exit 0
        else
            print_warning "OpenRank CLI $current_version is already installed"
            echo "Use --force to overwrite or specify a different version"
            exit 1
        fi
    fi

    # Create temporary directory
    local tmp_dir
    tmp_dir=$(mktemp -d)
    trap "rm -rf '$tmp_dir'" EXIT

    # Download URLs
    local base_url="https://github.com/${REPO}/releases/download/${version}"
    local binary_url="${base_url}/${asset_name}"
    local checksums_url="${base_url}/checksums.txt"

    print_status "Downloading $asset_name..."
    download_file "$binary_url" "$tmp_dir/$asset_name"

    # Download checksums
    print_status "Downloading checksums..."
    download_file "$checksums_url" "$tmp_dir/checksums.txt" || print_warning "Failed to download checksums"

    # Verify checksum
    if ! verify_checksum "$tmp_dir/$asset_name" "$tmp_dir/checksums.txt"; then
        print_error "Installation aborted due to checksum mismatch"
        exit 1
    fi

    # Make binary executable
    chmod +x "$tmp_dir/$asset_name"

    # Install binary
    if [ -w "$INSTALL_DIR" ]; then
        mv "$tmp_dir/$asset_name" "$INSTALL_DIR/$BINARY_NAME"
    else
        print_status "Installing to $INSTALL_DIR (requires sudo)..."
        sudo mv "$tmp_dir/$asset_name" "$INSTALL_DIR/$BINARY_NAME"
    fi

    print_success "OpenRank CLI $version installed successfully!"
    print_status "Binary location: $INSTALL_DIR/$BINARY_NAME"

    # Verify installation
    if command_exists "$BINARY_NAME"; then
        print_success "Installation verified. Run '$BINARY_NAME --help' to get started."
    else
        print_warning "Binary installed but not found in PATH. You may need to add $INSTALL_DIR to your PATH."
    fi
}

# Show usage information
show_usage() {
    cat << EOF
OpenRank CLI Installation Script

USAGE:
    $0 [OPTIONS]

OPTIONS:
    -v, --version VERSION    Install specific version (e.g., v1.0.0)
    -f, --force              Force installation even if already installed
    -d, --dir DIR           Installation directory (default: /usr/local/bin)
    -h, --help              Show this help message

EXAMPLES:
    $0                      # Install latest version
    $0 --version v1.0.0     # Install specific version
    $0 --force              # Force reinstall latest version
    $0 --dir ~/.local/bin   # Install to custom directory

EOF
}

# Parse command line arguments
main() {
    local version=""
    local force_install="false"

    while [[ $# -gt 0 ]]; do
        case $1 in
            -v|--version)
                version="$2"
                shift 2
                ;;
            -f|--force)
                force_install="true"
                shift
                ;;
            -d|--dir)
                INSTALL_DIR="$2"
                shift 2
                ;;
            -h|--help)
                show_usage
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                echo
                show_usage
                exit 1
                ;;
        esac
    done

    # Check if install directory exists
    if [ ! -d "$INSTALL_DIR" ]; then
        print_error "Installation directory does not exist: $INSTALL_DIR"
        exit 1
    fi

    install_openrank "$version" "$force_install"
}

# Run main function with all arguments
main "$@"
