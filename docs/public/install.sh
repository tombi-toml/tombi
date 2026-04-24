#!/bin/sh
set -e

# Tombi installation script
# Automatically installs tombi from GitHub releases based on detected architecture

# Temporary directory for installation
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

# Helper functions
print_step() {
	printf '\033[34m==>\033[m %s\n' "$1" >&2
}

print_error() {
	printf '\033[31mError:\033[m %s\n' "$1" >&2
}

print_success() {
	printf '\033[32mSuccess:\033[m %s\n' "$1" >&2
}

# Keep the 0.9.23 cutoff in sync with:
#   - xtask/src/command/dist.rs (UNIX_ARCHIVE_FORMAT_CUTOFF)
#   - editors/zed/src/lib.rs (TombiExtension::uses_legacy_unix_artifact)
#   - .github/workflows/release_cli_vscode.yml ("Set CLI artifact extension" step)
version_uses_legacy_unix_artifact() {
	if ! printf '%s' "$1" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; then
		return 1
	fi

	old_ifs=$IFS
	IFS=.
	set -- $1
	IFS=$old_ifs
	major=$1
	minor=$2
	patch=$3

	if [ "$major" -gt 0 ]; then
		return 1
	fi
	if [ "$minor" -lt 9 ]; then
		return 0
	fi
	if [ "$minor" -gt 9 ]; then
		return 1
	fi
	[ "$patch" -lt 23 ]
}

download_to_file() {
	URL="$1"
	OUTPUT_FILE="$2"

	if command -v curl >/dev/null 2>&1; then
		curl -L -f -s "${URL}" -o "${OUTPUT_FILE}"
	elif command -v wget >/dev/null 2>&1; then
		wget --tries=1 -q "${URL}" -O "${OUTPUT_FILE}"
	else
		print_error "Neither curl nor wget is installed. Please install one of them."
		exit 1
	fi
}

# Parse command line options
__SPECIFIED_VERSION=""
__SPECIFIED_INSTALL_DIR=""
while [ $# -gt 0 ]; do
	case $1 in
	--version)
		if [ "$2" != "latest" ]; then
			__SPECIFIED_VERSION="${2#v}"
		fi
		shift 2
		;;
	--install-dir)
		__SPECIFIED_INSTALL_DIR="$2"
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
		if [ "${ARCH}" = "aarch64" ]; then
			ARCH="aarch64"
			TARGET="${ARCH}-${OS}-musl"
		elif [ "${ARCH}" = "armv7l" ]; then
			ARCH="arm"
			TARGET="${ARCH}-${OS}-gnueabihf"
		else
			ARCH="x86_64"
			TARGET="${ARCH}-${OS}-musl"
		fi
		;;
	Darwin)
		OS="apple-darwin"
		if [ "${ARCH}" = "arm64" ]; then
			ARCH="aarch64"
		else
			ARCH="x86_64"
		fi
		TARGET="${ARCH}-${OS}"
		;;
	MINGW* | MSYS* | CYGWIN* | Windows_NT)
		OS="pc-windows-msvc"
		if [ "${ARCH}" = "aarch64" ]; then
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

artifact_extensions() {
	OS="$(uname -s)"

	case "${OS}" in
	MINGW* | MSYS* | CYGWIN* | Windows_NT)
		echo ".zip"
		;;
	*)
		if version_uses_legacy_unix_artifact "${VERSION}"; then
			echo ".gz .tar.gz"
		else
			echo ".tar.gz .gz"
		fi
		;;
	esac
}

find_extracted_binary() {
	EXE_NAME=$(get_exe_name)
	find "${TEMP_DIR}" -type f -name "${EXE_NAME}" | head -n 1
}

download_artifact() {
	print_step "Downloading tombi ${VERSION} (${TARGET})..."
	for extension in $(artifact_extensions); do
		CANDIDATE_URL="${RELEASE_BASE_URL}/v${VERSION}/tombi-cli-${VERSION}-${TARGET}${extension}"
		CANDIDATE_FILE="${TEMP_DIR}/tombi-${VERSION}${extension}"
		rm -f "${CANDIDATE_FILE}"

		print_step "Trying ${CANDIDATE_URL}"
		if download_to_file "${CANDIDATE_URL}" "${CANDIDATE_FILE}" && [ -s "${CANDIDATE_FILE}" ]; then
			ARTIFACT_EXTENSION="${extension}"
			DOWNLOAD_URL="${CANDIDATE_URL}"
			TEMP_FILE="${CANDIDATE_FILE}"
			return 0
		fi
	done

	return 1
}

# Create installation directories
create_install_dir() {
	if [ -n "${__SPECIFIED_INSTALL_DIR}" ]; then
		BIN_DIR="${__SPECIFIED_INSTALL_DIR}"
	else
		BIN_DIR="${HOME}/.local/bin"
	fi
	mkdir -p "${BIN_DIR}"

	if ! echo ":$PATH:" | grep -q ":${BIN_DIR}:"; then
		print_step "${BIN_DIR} is not in your PATH. Consider adding it to your shell configuration file."
	fi
}

# Download and install tombi
download_and_install() {
	if ! download_artifact; then
		print_error "Download failed. Please check the release assets for tombi ${VERSION} (${TARGET})."
		exit 1
	fi

	EXE_NAME=$(get_exe_name)
	if [ "${ARTIFACT_EXTENSION}" = ".zip" ]; then
		unzip -o "${TEMP_FILE}" -d "${TEMP_DIR}"
		EXTRACTED_FILE=$(find_extracted_binary)
	elif [ "${ARTIFACT_EXTENSION}" = ".tar.gz" ]; then
		tar -xzf "${TEMP_FILE}" -C "${TEMP_DIR}"
		EXTRACTED_FILE=$(find_extracted_binary)
	else
		gzip -d "${TEMP_FILE}" -f
		EXTRACTED_FILE="${TEMP_FILE%.gz}"
	fi

	if [ -z "${EXTRACTED_FILE}" ] || [ ! -f "${EXTRACTED_FILE}" ]; then
		print_error "Failed to locate ${EXE_NAME} in the downloaded archive."
		exit 1
	fi

	chmod +x "${EXTRACTED_FILE}"
	mv "${EXTRACTED_FILE}" "${BIN_DIR}/${EXE_NAME}"

	print_success "tombi ${VERSION} has been installed to ${BIN_DIR}/${EXE_NAME}"
}

# Version
LATEST_STABLE_VERSION="0.9.22"
if [ -n "${__SPECIFIED_VERSION}" ]; then
	VERSION="${__SPECIFIED_VERSION}"
	print_step "Using specified version: ${VERSION}"
else
	VERSION="${LATEST_STABLE_VERSION}"
	if ! printf '%s' "${VERSION}" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; then
		print_error "Invalid embedded stable version '${VERSION}'."
		exit 1
	fi
	print_step "Using latest version: ${VERSION}"
fi
RELEASE_BASE_URL="${TOMBI_RELEASE_BASE_URL:-https://github.com/tombi-toml/tombi/releases/download}"

# Get the executable name based on OS
get_exe_name() {
	OS="$(uname -s)"
	case "${OS}" in
	MINGW* | MSYS* | CYGWIN* | Windows_NT)
		echo "tombi.exe"
		;;
	*)
		echo "tombi"
		;;
	esac
}

# Main process
main() {
	print_step "Starting tombi installer..."
	detect_os_arch
	create_install_dir
	if ! download_and_install; then
		exit 1
	fi

	EXE_NAME=$(get_exe_name)
	INSTALLED_BINARY="${BIN_DIR}/${EXE_NAME}"

	# Verify installation
	if [ ! -f "${INSTALLED_BINARY}" ]; then
		print_error "Installation failed: ${INSTALLED_BINARY} not found."
		exit 1
	fi

	# Verify the binary can be found in PATH and executed
	if ! command -v "${EXE_NAME}" >/dev/null 2>&1; then
		print_error "Installation completed, but ${EXE_NAME} command not found in PATH."
		printf 'To run manually: \033[34m%s --help\033[m\n' "${INSTALLED_BINARY}" >&2
		exit 1
	fi

	if "${EXE_NAME}" --version >/dev/null 2>&1; then
		INSTALLED_VERSION=$("${EXE_NAME}" --version 2>&1 | head -n 1 | sed -E 's/tombi //; s/^v//' || echo "unknown")
		if [ "$INSTALLED_VERSION" != "$VERSION" ]; then
			print_error "Installed version mismatch: expected ${VERSION}, but got ${INSTALLED_VERSION}"
			exit 1
		fi
		printf 'Usage: \033[34m%s --help\033[m\n' "${EXE_NAME}" >&2
	else
		print_error "Installation completed, but ${EXE_NAME} cannot be executed."
		printf 'To run manually: \033[34m%s --help\033[m\n' "${INSTALLED_BINARY}" >&2
		exit 1
	fi
}

# Execute the script
main
