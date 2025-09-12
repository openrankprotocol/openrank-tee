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
SCRIPTS_BASE_URL="https://raw.githubusercontent.com/${REPO}/main/scripts"

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

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check glibc version compatibility
check_glibc_compatibility() {
    if ! command_exists ldd; then
        return 1  # Can't determine, assume incompatible
    fi

    local glibc_version
    glibc_version=$(ldd --version 2>&1 | head -n1 | grep -oE '[0-9]+\.[0-9]+' | head -n1)

    if [ -z "$glibc_version" ]; then
        return 1  # Can't determine version
    fi

    print_status "Detected glibc version: $glibc_version"

    # Parse version components
    local major minor
    major=$(echo "$glibc_version" | cut -d. -f1)
    minor=$(echo "$glibc_version" | cut -d. -f2)

    # Check if glibc is recent enough (2.32+)
    # This is a conservative threshold where most modern builds should work
    if [ "$major" -gt 2 ] || { [ "$major" -eq 2 ] && [ "$minor" -ge 32 ]; }; then
        return 0  # Compatible
    else
        return 1  # Incompatible
    fi
}

# Detect the best installation method
detect_installation_method() {
    local os
    os=$(uname -s)

    case "$os" in
        Linux*)
            print_status "Detected Linux system"

            # Check architecture
            local arch
            arch=$(uname -m)
            case "$arch" in
                x86_64|amd64|aarch64|arm64)
                    print_status "Architecture $arch supports static builds"
                    ;;
                *)
                    print_warning "Architecture $arch may not support static builds"
                    echo "regular"
                    return
                    ;;
            esac

            # Check glibc compatibility
            if check_glibc_compatibility; then
                print_status "glibc version is compatible with regular builds"
                echo "regular"
            else
                print_warning "glibc version may cause compatibility issues"
                print_status "Recommending static build for better compatibility"
                echo "static"
            fi
            ;;
        Darwin*)
            print_status "Detected macOS system - using regular installer"
            echo "regular"
            ;;
        CYGWIN*|MINGW*|MSYS*)
            print_status "Detected Windows system - using regular installer"
            echo "regular"
            ;;
        *)
            print_warning "Unknown operating system: $os"
            print_status "Defaulting to regular installer"
            echo "regular"
            ;;
    esac
}

# Download and execute the appropriate install script
run_installer() {
    local method="$1"
    shift  # Remove method from arguments, pass rest to installer

    local script_name
    case "$method" in
        static)
            script_name="install-static.sh"
            print_status "Using static installer (no external dependencies)"
            ;;
        regular)
            script_name="install-regular.sh"
            print_status "Using regular installer"
            ;;
        *)
            print_error "Unknown installation method: $method"
            exit 1
            ;;
    esac

    local script_url="${SCRIPTS_BASE_URL}/${script_name}"

    print_status "Downloading and executing $script_name..."
    print_status "URL: $script_url"

    if command_exists curl; then
        curl -fsSL "$script_url" | bash -s -- "$@"
    elif command_exists wget; then
        wget -qO- "$script_url" | bash -s -- "$@"
    else
        print_error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi
}

# Show usage information
show_usage() {
    cat << EOF
OpenRank CLI Smart Installation Script

This script automatically detects the best installation method for your system:
- Linux with old glibc: Uses static builds (no dependencies)
- Linux with modern glibc: Uses regular builds (smaller, faster)
- macOS/Windows: Uses regular builds (platform native)

USAGE:
    $0 [OPTIONS]

OPTIONS:
    --method METHOD         Force specific method: 'regular' or 'static'
    -v, --version VERSION   Install specific version (e.g., v1.0.0)
    -f, --force             Force installation even if already installed
    -d, --dir DIR          Installation directory (default: /usr/local/bin)
    -h, --help             Show this help message

EXAMPLES:
    $0                          # Auto-detect best method
    $0 --method static          # Force static build
    $0 --method regular         # Force regular build
    $0 --version v1.0.0         # Install specific version (auto-detect method)
    $0 --force --dir ~/.local/bin  # Force reinstall to custom directory

DETECTION LOGIC:
    - Linux + glibc < 2.32 → Static build (universal compatibility)
    - Linux + glibc ≥ 2.32 → Regular build (smaller, faster)
    - macOS/Windows → Regular build (platform native)
    - Unknown/unsupported → Regular build (fallback)

EOF
}

# Main function
main() {
    local forced_method=""
    local installer_args=()

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --method)
                forced_method="$2"
                if [[ "$forced_method" != "regular" && "$forced_method" != "static" ]]; then
                    print_error "Invalid method: $forced_method (must be 'regular' or 'static')"
                    exit 1
                fi
                shift 2
                ;;
            -h|--help)
                show_usage
                exit 0
                ;;
            *)
                # Pass through all other arguments to the installer
                installer_args+=("$1")
                shift
                ;;
        esac
    done

    print_status "OpenRank CLI Smart Installer"
    print_status "Analyzing system for optimal installation method..."
    echo

    # Determine installation method
    local method
    if [ -n "$forced_method" ]; then
        method="$forced_method"
        print_status "Using forced method: $method"
    else
        method=$(detect_installation_method)
        print_success "Recommended installation method: $method"
    fi

    echo
    print_status "Proceeding with $method installation..."
    echo

    # Run the appropriate installer with remaining arguments
    run_installer "$method" "${installer_args[@]}"
}

# Run main function with all arguments
main "$@"
