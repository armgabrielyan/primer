#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');
const https = require('https');
const os = require('os');
const path = require('path');
const zlib = require('zlib');

const REPO = 'armgabrielyan/primer';
const BINARY_NAME = 'primer';
const VERSION = require('../package.json').version;

function getTarget() {
  const platform = os.platform();
  const arch = os.arch();

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
    throw new Error(`Unsupported platform: ${key}`);
  }

  return target;
}

function getArchiveExt() {
  return os.platform() === 'win32' ? 'zip' : 'tar.gz';
}

function download(url) {
  return new Promise((resolve, reject) => {
    const handleResponse = (response) => {
      if (response.statusCode === 302 || response.statusCode === 301) {
        https.get(response.headers.location, handleResponse).on('error', reject);
        return;
      }

      if (response.statusCode !== 200) {
        reject(new Error(`Failed to download: ${response.statusCode}`));
        return;
      }

      const chunks = [];
      response.on('data', (chunk) => chunks.push(chunk));
      response.on('end', () => resolve(Buffer.concat(chunks)));
      response.on('error', reject);
    };

    https.get(url, handleResponse).on('error', reject);
  });
}

function extractTarGz(buffer, destDir) {
  const tarPath = path.join(os.tmpdir(), 'primer.tar');
  const decompressed = zlib.gunzipSync(buffer);
  fs.writeFileSync(tarPath, decompressed);
  execSync(`tar -xf "${tarPath}" -C "${destDir}"`, { stdio: 'inherit' });
  fs.unlinkSync(tarPath);
}

function extractZip(buffer, destDir) {
  const zipPath = path.join(os.tmpdir(), 'primer.zip');
  fs.writeFileSync(zipPath, buffer);
  execSync(`powershell -Command "Expand-Archive -Path '${zipPath}' -DestinationPath '${destDir}' -Force"`, {
    stdio: 'inherit',
  });
  fs.unlinkSync(zipPath);
}

async function install() {
  console.log('Installing primer...');

  try {
    const target = getTarget();
    const ext = getArchiveExt();
    const archiveName = `primer-${VERSION}-${target}.${ext}`;
    const url = `https://github.com/${REPO}/releases/download/v${VERSION}/${archiveName}`;

    console.log(`Downloading ${archiveName}...`);
    const buffer = await download(url);

    const binDir = path.join(__dirname, '..', 'bin');
    if (!fs.existsSync(binDir)) {
      fs.mkdirSync(binDir, { recursive: true });
    }

    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'primer-'));

    console.log('Extracting...');
    if (ext === 'zip') {
      extractZip(buffer, tmpDir);
    } else {
      extractTarGz(buffer, tmpDir);
    }

    const binaryExt = os.platform() === 'win32' ? '.exe' : '';
    const srcBinary = path.join(tmpDir, BINARY_NAME + binaryExt);
    const destBinary = path.join(binDir, BINARY_NAME + binaryExt);

    fs.copyFileSync(srcBinary, destBinary);

    if (os.platform() !== 'win32') {
      fs.chmodSync(destBinary, 0o755);
    }

    fs.rmSync(tmpDir, { recursive: true, force: true });

    console.log('primer installed successfully.');
  } catch (error) {
    console.error('Failed to install primer:', error.message);
    console.error('');
    console.error('You can install from source instead:');
    console.error('  cargo install primer');
    process.exit(1);
  }
}

install();
