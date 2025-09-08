#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BINARY_NAME="openrank"
DEFAULT_INSTALL_DIRS=("/usr/local/bin" "$HOME/.local/bin" "/usr/bin")

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

# Find installed binary
find_binary() {
    # First check if it's in PATH
    if command_exists "$BINARY_NAME"; then
        which "$BINARY_NAME" 2>/dev/null
        return 0
    fi

    # Check common installation directories
    for dir in "${DEFAULT_INSTALL_DIRS[@]}"; do
        if [ -f "$dir/$BINARY_NAME" ]; then
            echo "$dir/$BINARY_NAME"
            return 0
        fi
    done

    return 1
}

# Remove binary file
remove_binary() {
    local binary_path="$1"
    local dir=$(dirname "$binary_path")

    print_status "Removing binary: $binary_path"

    if [ -w "$dir" ]; then
        rm -f "$binary_path"
    else
        print_status "Removing binary (requires sudo)..."
        sudo rm -f "$binary_path"
    fi

    if [ -f "$binary_path" ]; then
        print_error "Failed to remove binary: $binary_path"
        return 1
    else
        print_success "Binary removed: $binary_path"
        return 0
    fi
}

# Main uninstall function
uninstall_openrank() {
    local force_remove="$1"

    print_status "Starting OpenRank CLI uninstallation..."

    # Find binary
    local binary_path
    binary_path=$(find_binary)

    if [ $? -ne 0 ]; then
        if [ "$force_remove" = "true" ]; then
            print_warning "OpenRank CLI not found, but continuing with cleanup..."
        else
            print_error "OpenRank CLI is not installed or not found in common locations"
            print_status "Use --force to remove any remaining files"
            exit 1
        fi
    else
        print_status "Found OpenRank CLI at: $binary_path"

        # Get version before removing
        local version
        version=$("$binary_path" --version 2>/dev/null | grep -oE 'v[0-9]+\.[0-9]+\.[0-9]+' || echo "unknown")
        print_status "Currently installed version: $version"

        # Confirm removal
        if [ "$force_remove" != "true" ]; then
            echo
            read -p "Are you sure you want to uninstall OpenRank CLI? (y/N): " confirm
            case $confirm in
                [Yy]* )
                    print_status "Proceeding with uninstallation..."
                    ;;
                * )
                    print_status "Uninstallation cancelled"
                    exit 0
                    ;;
            esac
        fi

        # Remove binary
        if ! remove_binary "$binary_path"; then
            exit 1
        fi
    fi

    # Clean up additional files (if any)
    local cleanup_dirs=(
        "$HOME/.openrank"
        "$HOME/.config/openrank"
        "$HOME/.cache/openrank"
    )

    for cleanup_dir in "${cleanup_dirs[@]}"; do
        if [ -d "$cleanup_dir" ]; then
            print_status "Found configuration directory: $cleanup_dir"
            if [ "$force_remove" = "true" ]; then
                rm -rf "$cleanup_dir"
                print_success "Removed: $cleanup_dir"
            else
                read -p "Remove configuration directory $cleanup_dir? (y/N): " confirm
                case $confirm in
                    [Yy]* )
                        rm -rf "$cleanup_dir"
                        print_success "Removed: $cleanup_dir"
                        ;;
                    * )
                        print_status "Kept: $cleanup_dir"
                        ;;
                esac
            fi
        fi
    done

    print_success "OpenRank CLI uninstallation completed!"

    # Verify removal
    if command_exists "$BINARY_NAME"; then
        print_warning "Binary still found in PATH. You may need to restart your terminal or check other installation locations."
    else
        print_success "OpenRank CLI successfully removed from your system."
    fi
}

# Show usage information
show_usage() {
    cat << EOF
OpenRank CLI Uninstallation Script

USAGE:
    $0 [OPTIONS]

OPTIONS:
    -f, --force     Force removal without prompts and clean up all files
    -h, --help      Show this help message

EXAMPLES:
    $0              # Interactive uninstall
    $0 --force      # Force uninstall without prompts

This script will:
1. Locate the OpenRank CLI binary
2. Remove the binary file
3. Optionally clean up configuration directories
4. Verify the removal

EOF
}

# Parse command line arguments
main() {
    local force_remove="false"

    while [[ $# -gt 0 ]]; do
        case $1 in
            -f|--force)
                force_remove="true"
                shift
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

    uninstall_openrank "$force_remove"
}

# Run main function with all arguments
main "$@"
