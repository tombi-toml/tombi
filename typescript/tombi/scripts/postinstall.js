#!/usr/bin/env node

const axios = require('axios');
const fs = require('fs');
const path = require('path');
const os = require('os');
const zlib = require('zlib');

// Configuration for downloading binary from GitHub releases
const REPO = 'tombi-toml/tombi';
const BINARY_NAME = 'tombi';
const BIN_PATH = path.join(__dirname, '..', 'bin');

async function main() {
  try {
    // Get package version
    const packageJson = require('../package.json');
    const currentVersion = packageJson.version;

    // Get version to use
    const version = currentVersion === '0.0.0' ? await getLatestVersion() : currentVersion;
    console.log(`🦅 Installing tombi v${version}...`);

    // Determine platform and architecture
    const { target, artifactExtension } = detectOsArch();
    console.log(`Detected system: ${target}`);

    // Build download URL
    const downloadUrl = `https://github.com/${REPO}/releases/download/v${version}/tombi-cli-${version}-${target}${artifactExtension}`;

    // Create bin directory
    if (!fs.existsSync(BIN_PATH)) {
      fs.mkdirSync(BIN_PATH, { recursive: true });
    }

    // Download and extract tarball
    await downloadAndExtract(downloadUrl, artifactExtension);

    // Add execute permission to binary
    const binaryPath = path.join(BIN_PATH, BINARY_NAME);
    fs.chmodSync(binaryPath, 0o755);

    console.log(`✅ Installation of tombi v${version} completed!`);
    console.log(`Binary location: ${binaryPath}`);

  } catch (error) {
    console.error('❌ An error occurred during installation:');
    console.error(error.message || error);
    process.exit(1);
  }
}

async function getLatestVersion() {
  try {
    const response = await axios.get(`https://api.github.com/repos/${REPO}/releases/latest`);
    return response.data.tag_name.replace('v', '');
  } catch (error) {
    throw new Error(`Failed to get latest version: ${error.message}`);
  }
}

function detectOsArch() {
  const platform = os.platform();
  const arch = os.arch();
  let target;
  let artifactExtension;

  switch (platform) {
    case 'darwin':
      if (arch === 'arm64') {
        target = 'aarch64-apple-darwin';
      } else {
        target = 'x86_64-apple-darwin';
      }
      artifactExtension = '.gz';
      break;
    case 'linux':
      if (arch === 'arm64') {
        target = 'aarch64-unknown-linux-musl';
      } else if (arch === 'arm') {
        target = 'arm-unknown-linux-gnueabihf';
      } else {
        target = 'x86_64-unknown-linux-musl';
      }
      artifactExtension = '.gz';
      break;
    case 'win32':
      if (arch === 'arm64') {
        target = 'aarch64-pc-windows-msvc';
      } else {
        target = 'x86_64-pc-windows-msvc';
      }
      artifactExtension = '.zip';
      break;
    default:
      throw new Error(`Unsupported platform: ${platform}`);
  }

  return { target, artifactExtension };
}

async function downloadAndExtract(url, artifactExtension) {
  const tempFile = path.join(os.tmpdir(), `tombi-${Date.now()}${artifactExtension}`);

  try {
    // Download
    console.log(`📦 Download binary from ${url}`);
    const response = await axios({
      method: 'get',
      url: url,
      responseType: 'stream'
    });

    const writer = fs.createWriteStream(tempFile);
    response.data.pipe(writer);

    await new Promise((resolve, reject) => {
      writer.on('finish', resolve);
      writer.on('error', reject);
    });

    // Extract
    if (artifactExtension === '.zip') {
      // Handle zip file for Windows
      const AdmZip = require('adm-zip');
      const zip = new AdmZip(tempFile);
      zip.extractAllTo(BIN_PATH, true);
    } else {
      // Handle tar.gz file for Linux/macOS
      const binaryPath = path.join(BIN_PATH, BINARY_NAME);
      const inp = fs.createReadStream(tempFile);
      const out = fs.createWriteStream(binaryPath);
      await new Promise((resolve, reject) => {
        inp.pipe(zlib.createGunzip()).pipe(out);
        out.on('finish', resolve);
        out.on('error', reject);
      });
    }

  } catch (error) {
    throw new Error(`Failed to download or extract: ${error.message}`);
  } finally {
    // Clean up temporary file
    if (fs.existsSync(tempFile)) {
      fs.unlinkSync(tempFile);
    }
  }
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
