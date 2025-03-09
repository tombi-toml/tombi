#!/bin/bash
set -e

# Tombi installation script
# Automatically installs tombi from GitHub releases based on detected architecture

# Color settings
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Temporary directory for installation
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

# Helper functions
print_step() {
    echo "${BLUE}==>${NC} $1"
}

print_error() {
    echo "${RED}Error:${NC} $1"
}

print_success() {
    echo "${GREEN}Success:${NC} $1"
}

# Parse command line options
while [[ $# -gt 0 ]]; do
    case $1 in
    --version)
        SPECIFIED_VERSION="$2"
        shift 2
        ;;
    *)
        print_error "Unknown option: $1"
        exit 1
        ;;
    esac
done

# Detect OS and architecture
detect_os_arch() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "${OS}" in
    Linux)
        OS="unknown-linux"
        if [[ "${ARCH}" == "aarch64" ]]; then
            ARCH="aarch64"
            TARGET="${ARCH}-${OS}-gnu"
        elif [[ "${ARCH}" == "armv7l" ]]; then
            ARCH="arm"
            TARGET="${ARCH}-${OS}-gnueabihf"
        else
            ARCH="x86_64"
            TARGET="${ARCH}-${OS}-gnu"
        fi
        ;;
    Darwin)
        OS="apple-darwin"
        if [[ "${ARCH}" == "arm64" ]]; then
            ARCH="aarch64"
        else
            ARCH="x86_64"
        fi
        TARGET="${ARCH}-${OS}"
        ;;
    MINGW* | MSYS* | CYGWIN* | Windows_NT)
        OS="pc-windows-msvc"
        if [[ "${ARCH}" == "aarch64" ]]; then
            ARCH="aarch64"
        else
            ARCH="x86_64"
        fi
        TARGET="${ARCH}-${OS}"
        ;;
    *)
        print_error "Unsupported OS: ${OS}"
        exit 1
        ;;
    esac

    print_step "Detected system: ${TARGET}"
}

artifact_extension() {
    OS="$(uname -s)"

    case "${OS}" in
    MINGW* | MSYS* | CYGWIN* | Windows_NT)
        echo ".zip"
        ;;
    *)
        echo ".gz"
        ;;
    esac
}

# Create installation directories
create_install_dir() {
    BIN_DIR="${HOME}/.local/bin"
    mkdir -p "${BIN_DIR}"

    if [[ ! ":$PATH:" == *":${BIN_DIR}:"* ]]; then
        print_step "${BIN_DIR} is not in your PATH. Consider adding it to your shell configuration file."
    fi
}

# Download and install tombi
download_and_install() {
    DOWNLOAD_URL="${GITHUB_RELEASE_URL}/v${VERSION}/tombi-cli-${VERSION}-${TARGET}${ARTIFACT_EXTENSION}"
    TEMP_FILE="${TEMP_DIR}/tombi-${VERSION}${ARTIFACT_EXTENSION}"

    print_step "Download from ${DOWNLOAD_URL}"
    print_step "Downloading tombi v${VERSION} (${TARGET})..."

    if command -v curl >/dev/null 2>&1; then
        if ! curl -L -f -s "${DOWNLOAD_URL}" -o "${TEMP_FILE}"; then
            print_error "Download failed. Please check the URL: ${DOWNLOAD_URL}"
            exit 1
        fi
    elif command -v wget >/dev/null 2>&1; then
        if ! wget --tries=1 -q "${DOWNLOAD_URL}" -O "${TEMP_FILE}"; then
            print_error "Download failed. Please check the URL: ${DOWNLOAD_URL}"
            exit 1
        fi
    else
        print_error "Neither curl nor wget is installed. Please install one of them."
        exit 1
    fi

    if [ ! -f "${TEMP_FILE}" ] || [ ! -s "${TEMP_FILE}" ]; then
        print_error "Download failed. Please check the URL: ${DOWNLOAD_URL}"
        exit 1
    fi

    if [[ "${ARTIFACT_EXTENSION}" == ".zip" ]]; then
        unzip -o "${TEMP_FILE}" -d "${TEMP_DIR}"
    elif [[ "${ARTIFACT_EXTENSION}" == ".gz" ]]; then
        gzip -d "${TEMP_FILE}" -f
    fi

    EXTRACTED_FILE="${TEMP_DIR}/tombi-${VERSION}"
    chmod +x "${EXTRACTED_FILE}"
    mv "${EXTRACTED_FILE}" "${BIN_DIR}/tombi"

    print_success "tombi v${VERSION} has been installed to ${BIN_DIR}/tombi"
}

# Version
REPO="tombi-toml/tombi"
GITHUB_RELEASE_URL="https://github.com/${REPO}/releases/download"
if [ -n "${SPECIFIED_VERSION}" ]; then
    VERSION="${SPECIFIED_VERSION}"
    print_step "Using specified version: v${VERSION}"
else
    VERSION=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"v([^"]+)".*/\1/')
    print_step "Using latest version: v${VERSION}"
fi
ARTIFACT_EXTENSION=$(artifact_extension)

# Main process
main() {
    print_step "Starting tombi installer..."
    detect_os_arch
    create_install_dir
    if ! download_and_install; then
        exit 1
    fi

    # Verify installation
    if command -v tombi >/dev/null 2>&1; then
        if tombi --version >/dev/null 2>&1; then
            INSTALLED_VERSION=$(tombi --version 2>&1 | head -n 1 | sed 's/tombi //g' || echo "unknown")
            if [ "v${INSTALLED_VERSION}" != "v${VERSION}" ]; then
                print_error "Installed version mismatch: expected v${VERSION}, but got v${INSTALLED_VERSION}"
                exit 1
            fi
            echo "Usage: ${GREEN}tombi --help${NC}"
        else
            print_error "Installation completed, but tombi command cannot be executed. "
            echo "To run manually: ${GREEN}${BIN_DIR}/tombi --help${NC}"
            exit 1
        fi
    else
        print_error "Installation completed, but tombi command not found. Please check your PATH settings."
        echo "To run manually: ${GREEN}${BIN_DIR}/tombi --help${NC}"
        exit 1
    fi
}

# Execute the script
main
