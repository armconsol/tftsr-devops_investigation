#!/usr/bin/env node

import { execSync } from 'child_process';
import { readFileSync, writeFileSync, existsSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const projectRoot = resolve(__dirname, '..');

/**
 * Validate version is semver-compliant, optionally with a prerelease suffix
 * (X.Y.Z or X.Y.Z-<prerelease>, e.g. 3.1.0-beta.9).
 */
export function isValidVersion(version) {
  return /^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$/.test(version);
}

/** Strip a leading "v" from a tag name. */
export function stripV(tag) {
  return tag.replace(/^v/, '');
}

/**
 * Resolve the version to embed in the build, in order of precedence:
 * explicit CLI argument > $RELEASE_TAG > `git describe` > package.json.
 * Each source is validated and skipped (with a warning) if malformed, so a
 * bad explicit argument doesn't silently poison the build the way an
 * unvalidated `git describe` result historically did.
 */
export function resolveVersion({ argVersion, releaseTagEnv, gitDescribe, packageJsonFallback }) {
  if (argVersion) {
    const v = stripV(argVersion);
    if (isValidVersion(v)) return v;
    console.warn(`Invalid explicit version "${argVersion}", ignoring`);
  }

  if (releaseTagEnv) {
    const v = stripV(releaseTagEnv);
    if (isValidVersion(v)) return v;
    console.warn(`Invalid RELEASE_TAG "${releaseTagEnv}", ignoring`);
  }

  try {
    const v = stripV(gitDescribe());
    if (isValidVersion(v)) return v;
    console.warn(`Invalid version "${v}" from git describe, ignoring`);
  } catch {
    // No tags reachable (shallow clone, fresh repo, etc.) — fall through.
  }

  return packageJsonFallback();
}

function validateGitRepo(root) {
  if (!existsSync(resolve(root, '.git'))) {
    throw new Error(`Not a Git repository: ${root}`);
  }
}

function gitDescribeTag() {
  validateGitRepo(projectRoot);
  return execSync('git describe --tags --abbrev=0', {
    encoding: 'utf-8',
    cwd: projectRoot,
    shell: false,
  }).trim();
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
  let replaced = false;

  for (const line of lines) {
    if (!replaced && line.match(/^\s*version\s*=\s*"/)) {
      output.push(`version = "${version}"`);
      replaced = true;
    } else {
      output.push(line);
    }
  }

  writeFileSync(fullPath, output.join('\n') + '\n', 'utf-8');
  console.log(`✓ Updated ${path} to ${version}`);
}

function updateJSON(path, version) {
  const fullPath = resolve(projectRoot, path);
  if (!existsSync(fullPath)) {
    throw new Error(`File not found: ${fullPath}`);
  }

  const content = readFileSync(fullPath, 'utf-8');
  const json = JSON.parse(content);
  json.version = version;

  writeFileSync(fullPath, JSON.stringify(json, null, 2) + '\n', 'utf-8');
  console.log(`✓ Updated ${path} to ${version}`);
}

function main() {
  const version = resolveVersion({
    argVersion: process.argv[2],
    releaseTagEnv: process.env.RELEASE_TAG,
    gitDescribe: gitDescribeTag,
    packageJsonFallback: getFallbackVersion,
  });
  console.log(`Setting version to: ${version}`);

  updatePackageJson(version);
  updateTOML('src-tauri/Cargo.toml', version);
  updateJSON('src-tauri/tauri.conf.json', version);

  console.log(`✓ All version fields updated to ${version}`);
}

// Only run when executed directly (`node update-version.mjs`), not when
// imported for its pure functions (e.g. by tests).
if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}
