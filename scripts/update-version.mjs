#!/usr/bin/env node

import { execSync } from 'child_process';
import { readFileSync, writeFileSync, existsSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const projectRoot = resolve(__dirname, '..');

function getVersionFromGit() {
  try {
    const output = execSync('git describe --tags --abbrev=0', { 
      encoding: 'utf-8',
      cwd: projectRoot
    });
    const version = output.trim().replace(/^v/, '');
    if (version) {
      return version;
    }
  } catch (e) {
    console.warn('Failed to get version from Git tags, using fallback');
  }
  return '0.2.50';
}

function updateFile(filePath, updater) {
  const fullPath = resolve(projectRoot, filePath);
  if (!existsSync(fullPath)) {
    throw new Error(`File not found: ${fullPath}`);
  }
  const content = readFileSync(fullPath, 'utf-8');
  const updated = updater(content);
  writeFileSync(fullPath, updated, 'utf-8');
  console.log(`✓ Updated ${filePath}`);
}

const version = getVersionFromGit();
console.log(`Setting version to: ${version}`);

// Update Cargo.toml (Rust)
updateFile('src-tauri/Cargo.toml', (content) => {
  return content.replace(/version = "([^"]+)"/, `version = "${version}"`);
});

// Update package.json (Frontend)
updateFile('package.json', (content) => {
  return content.replace(/"version": "([^"]+)"/, `"version": "${version}"`);
});

// Update tauri.conf.json
updateFile('src-tauri/tauri.conf.json', (content) => {
  return content.replace(/"version": "([^"]+)"/, `"version": "${version}"`);
});

console.log(`✓ All version fields updated to ${version}`);
