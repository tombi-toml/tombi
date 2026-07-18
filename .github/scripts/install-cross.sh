#!/usr/bin/env bash
# Download a pinned cross-rs release, verify its SHA256 checksum, and add it to PATH.
set -euo pipefail

CROSS_VERSION="${CROSS_VERSION:-0.2.5}"
CROSS_SHA256="${CROSS_SHA256:-642375d1bcf3bd88272c32ba90e999f3d983050adf45e66bd2d3887e8e838bad}"

work_dir="$(mktemp -d)"
trap 'rm -rf "${work_dir}"' EXIT

url="https://github.com/cross-rs/cross/releases/download/v${CROSS_VERSION}/cross-x86_64-unknown-linux-gnu.tar.gz"
curl -fsSL "${url}" -o "${work_dir}/cross.tar.gz"
echo "${CROSS_SHA256}  ${work_dir}/cross.tar.gz" | sha256sum --check || {
	echo "::error::cross v${CROSS_VERSION} tarball failed sha256 verification (expected ${CROSS_SHA256})"
	exit 1
}

install_dir="${HOME}/.cross-bin"
mkdir -p "${install_dir}"
tar -xzf "${work_dir}/cross.tar.gz" -C "${install_dir}" cross cross-util
echo "${install_dir}" >>"${GITHUB_PATH}"
