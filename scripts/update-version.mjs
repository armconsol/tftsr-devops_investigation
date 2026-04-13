#!/usr/bin/env node

import { execSync } from 'child_process';
import { readFileSync, writeFileSync, existsSync, mkdirSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const projectRoot = resolve(__dirname, '..');

/**
 * Validate version is semver-compliant (X.Y.Z)
 */
function isValidSemver(version) {
  return /^[0-9]+\.[0-9]+\.[0-9]+$/.test(version);
}

function validateGitRepo(root) {
  if (!existsSync(resolve(root, '.git'))) {
    throw new Error(`Not a Git repository: ${root}`);
  }
}

function getVersionFromGit() {
  validateGitRepo(projectRoot);
  try {
    const output = execSync('git describe --tags --abbrev=0', { 
      encoding: 'utf-8',
      cwd: projectRoot,
      shell: false
    });
    let version = output.trim();
    
    // Remove v prefix
    version = version.replace(/^v/, '');
    
    // Validate it's a valid semver
    if (!isValidSemver(version)) {
      const pkgJsonVersion = getFallbackVersion();
      console.warn(`Invalid version format "${version}" from git describe, using package.json fallback: ${pkgJsonVersion}`);
      return pkgJsonVersion;
    }
    
    return version;
  } catch (e) {
    const pkgJsonVersion = getFallbackVersion();
    console.warn(`Failed to get version from Git tags, using package.json fallback: ${pkgJsonVersion}`);
    return pkgJsonVersion;
  }
}

function getFallbackVersion() {
  const pkgPath = resolve(projectRoot, 'package.json');
  if (!existsSync(pkgPath)) {
    return '0.2.50';
  }
  try {
    const content = readFileSync(pkgPath, 'utf-8');
    const json = JSON.parse(content);
    return json.version || '0.2.50';
  } catch {
    return '0.2.50';
  }
}

function updatePackageJson(version) {
  const fullPath = resolve(projectRoot, 'package.json');
  if (!existsSync(fullPath)) {
    throw new Error(`File not found: ${fullPath}`);
  }
  
  const content = readFileSync(fullPath, 'utf-8');
  const json = JSON.parse(content);
  json.version = version;
  
  // Write with 2-space indentation
  writeFileSync(fullPath, JSON.stringify(json, null, 2) + '\n', 'utf-8');
  console.log(`✓ Updated package.json to ${version}`);
}

function updateTOML(path, version) {
  const fullPath = resolve(projectRoot, path);
  if (!existsSync(fullPath)) {
    throw new Error(`File not found: ${fullPath}`);
  }
  
  const content = readFileSync(fullPath, 'utf-8');
  const lines = content.split('\n');
  const output = [];
  
  for (const line of lines) {
    if (line.match(/^\s*version\s*=\s*"/)) {
      output.push(`version = "${version}"`);
    } else {
      output.push(line);
    }
  }
  
  writeFileSync(fullPath, output.join('\n') + '\n', 'utf-8');
  console.log(`✓ Updated ${path} to ${version}`);
}

const version = getVersionFromGit();
console.log(`Setting version to: ${version}`);

updatePackageJson(version);
updateTOML('src-tauri/Cargo.toml', version);
updateTOML('src-tauri/tauri.conf.json', version);

console.log(`✓ All version fields updated to ${version}`);
