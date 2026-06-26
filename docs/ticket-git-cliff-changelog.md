# feat: Automated Changelog via git-cliff

## Description

Introduces automated changelog generation using **git-cliff**, a tool that parses
conventional commits and produces formatted Markdown changelogs.

Previously, every Gitea release body contained only the static text `"Release vX.Y.Z"`.
With this change, releases display a categorised, human-readable list of all commits
since the previous version.

**Root cause / motivation:** No changelog tooling existed. The project follows
Conventional Commits throughout but the information was never surfaced to end-users.

**Files changed:**
- `cliff.toml` (new) — git-cliff configuration; defines commit parsers, ignored tags,
  output template, and which commit types appear in the changelog
- `CHANGELOG.md` (new) — bootstrapped from all existing tags; maintained by CI going forward
- `.gitea/workflows/auto-tag.yml` — new `changelog` job that runs after `autotag`
- `docs/wiki/CICD-Pipeline.md` — "Changelog Generation" section added

## Acceptance Criteria

- [ ] `cliff.toml` present at repo root with working Tera template
- [ ] `CHANGELOG.md` present at repo root, bootstrapped from all existing semver tags
- [ ] `changelog` job in `auto-tag.yml` runs after `autotag` (parallel with build jobs)
- [ ] Each Gitea release body shows grouped conventional-commit entries instead of
      static `"Release vX.Y.Z"`
- [ ] `CHANGELOG.md` committed to master on every release with `[skip ci]` suffix
      (no infinite re-trigger loop)
- [ ] `CHANGELOG.md` uploaded as a downloadable release asset
- [ ] CI/chore/build/test/style commits excluded from changelog output
- [ ] `docs/wiki/CICD-Pipeline.md` documents the changelog generation process

## Work Implemented

### `cliff.toml`
- Tera template with proper whitespace control (`-%}` / `{%- `) for clean output
- Included commit types: `feat`, `fix`, `perf`, `docs`, `refactor`
- Excluded commit types: `ci`, `chore`, `build`, `test`, `style`
- `ignore_tags = "rc|alpha|beta"` — pre-release tags excluded from version boundaries
- `filter_unconventional = true` — non-conventional commits dropped silently
- `sort_commits = "oldest"` — chronological order within each version

### `CHANGELOG.md`
- Bootstrapped locally using git-cliff v2.7.0 (aarch64 musl binary)
- Covers all tagged versions from `v0.1.0` through `v0.2.49` plus `[Unreleased]`
- 267 lines covering the full project history

### `.gitea/workflows/auto-tag.yml` — `changelog` job
- `needs: autotag` — waits for the new tag to exist before running
- Full history clone: `git fetch --tags --depth=2147483647` so git-cliff can resolve
  all version boundaries
- git-cliff v2.7.0 downloaded as a static x86_64 musl binary (~5 MB); no custom
  image required
- Generates full `CHANGELOG.md` and per-release notes (`--latest --strip all`)
- PATCHes the Gitea release body via API with JSON-safe escaping (`jq -Rs .`)
- Commits `CHANGELOG.md` to master with `[skip ci]` to prevent workflow re-trigger
- Deletes any existing `CHANGELOG.md` asset before re-uploading (rerun-safe)
- Runs in parallel with all build jobs — no added wall-clock latency

### `docs/wiki/CICD-Pipeline.md`
- Added "Changelog Generation" section before "Known Issues & Fixes"
- Describes the five-step process, cliff.toml settings, and loop prevention mechanism

## Testing Needed

- [ ] Merge this PR to master; verify `changelog` CI job succeeds in Gitea Actions
- [ ] Check Gitea release body for the new version tag — should show grouped commit list
- [ ] Verify `CHANGELOG.md` was committed to master (check git log after CI runs)
- [ ] Verify `CHANGELOG.md` appears as a downloadable asset on the release page
- [ ] Push a subsequent commit to master; confirm the `[skip ci]` CHANGELOG commit does
      NOT trigger a second run of `auto-tag.yml`
- [ ] Confirm CI/chore commits are absent from the release body
