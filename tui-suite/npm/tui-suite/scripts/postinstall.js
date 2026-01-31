#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');

const PACKAGE_VERSION = require('../package.json').version;
const REPO = 'utsavsharma/GSuiteTUI';

// Map Node.js platform/arch to Rust target
function getTarget() {
  const platform = process.platform;
  const arch = process.arch;

  const targets = {
    'darwin-x64': 'x86_64-apple-darwin',
    'darwin-arm64': 'aarch64-apple-darwin',
    'linux-x64': 'x86_64-unknown-linux-gnu',
    'linux-arm64': 'aarch64-unknown-linux-gnu',
    'win32-x64': 'x86_64-pc-windows-msvc',
  };

  const key = `${platform}-${arch}`;
  const target = targets[key];

  if (!target) {
    console.error(`Unsupported platform: ${key}`);
    console.error('Supported platforms: darwin-x64, darwin-arm64, linux-x64, linux-arm64, win32-x64');
    process.exit(1);
  }

  return { target, platform, arch };
}

function getBinaryName(platform) {
  return platform === 'win32' ? 'tui-suite.exe' : 'tui-suite';
}

function downloadFile(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);

    const request = (url) => {
      https.get(url, (response) => {
        // Handle redirects
        if (response.statusCode === 302 || response.statusCode === 301) {
          request(response.headers.location);
          return;
        }

        if (response.statusCode !== 200) {
          reject(new Error(`Failed to download: ${response.statusCode}`));
          return;
        }

        response.pipe(file);
        file.on('finish', () => {
          file.close();
          resolve();
        });
      }).on('error', (err) => {
        fs.unlink(dest, () => {});
        reject(err);
      });
    };

    request(url);
  });
}

async function main() {
  const { target, platform } = getTarget();
  const binaryName = getBinaryName(platform);
  const binDir = path.join(__dirname, '..', 'bin');
  const binaryPath = path.join(binDir, binaryName === 'tui-suite.exe' ? 'tui-suite.exe' : 'tui-suite');

  // Check if binary already exists
  if (fs.existsSync(binaryPath)) {
    console.log('tui-suite binary already exists, skipping download');
    return;
  }

  // Ensure bin directory exists
  if (!fs.existsSync(binDir)) {
    fs.mkdirSync(binDir, { recursive: true });
  }

  const downloadUrl = `https://github.com/${REPO}/releases/download/v${PACKAGE_VERSION}/tui-suite-${target}.tar.gz`;
  const tarPath = path.join(binDir, 'tui-suite.tar.gz');

  console.log(`Downloading tui-suite for ${target}...`);
  console.log(`URL: ${downloadUrl}`);

  try {
    await downloadFile(downloadUrl, tarPath);

    // Extract the tarball
    console.log('Extracting...');
    execSync(`tar -xzf "${tarPath}" -C "${binDir}"`, { stdio: 'inherit' });

    // Clean up tarball
    fs.unlinkSync(tarPath);

    // Make binary executable (Unix only)
    if (platform !== 'win32') {
      fs.chmodSync(binaryPath, 0o755);
    }

    console.log('tui-suite installed successfully!');
  } catch (error) {
    console.error('Failed to download tui-suite binary:', error.message);
    console.error('');
    console.error('You can manually build from source:');
    console.error('  git clone https://github.com/utsavsharma/GSuiteTUI');
    console.error('  cd GSuiteTUI/tui-suite');
    console.error('  cargo build --release');
    process.exit(1);
  }
}

main();
