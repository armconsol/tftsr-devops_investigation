# Plan: Disable Gitea Workflows on Beta and Master

## Goal
Permanently disable the following 4 Gitea workflows on both `beta` and `master` branches **without triggering new builds**:
1. `release-mirror-github.yml`
2. `sync-beta.yml`
3. `sync-inbound.yml`
4. `sync-outbound.yml`

## Context
- **Gitea Actions** (not GitHub Actions): Gitea only recognizes workflow files with `.yml` or `.yaml` extensions in `.gitea/workflows/` or `.github/workflows/`.
- **Prior attempt failed**: Commit `b67c48d3` added `if: false` at the job level, but Gitea still registers and displays workflow runs (job just shows as skipped).
- **Build suppression**: Gitea honors `[skip ci]` in commit messages to suppress all workflow runs. This is the repo-sanctioned mechanism (used throughout `auto-tag.yml`, `sync-beta.yml` comments).
- **File distribution**:
  - `beta`: has all 4 target files
  - `master`: has only `sync-beta.yml` and `sync-outbound.yml` (no `release-mirror-github.yml` or `sync-inbound.yml`)

## Implementation Strategy

### Method: Rename to `.yml.disabled`
- Rename each target file by appending `.disabled` to the filename (e.g., `sync-beta.yml` → `sync-beta.yml.disabled`).
- Gitea only parses `.yml`/`.yaml` files; renaming de-registers the workflow entirely.
- Reversible: rename back to re-enable.

### Branch Coverage
Apply the renames on **both branches**:
1. **beta branch**: Rename all 4 files
2. **master branch**: Rename `sync-beta.yml` and `sync-outbound.yml` only

### Commit & Push (No Builds)
- Use `[skip ci]` in the commit message to suppress workflow runs on push.
- Commit message format: `ci: disable [workflow-name] workflow [skip ci]`

## Task List

### Beta Branch
1. Rename `.gitea/workflows/release-mirror-github.yml` → `.gitea/workflows/release-mirror-github.yml.disabled`
2. Rename `.gitea/workflows/sync-beta.yml` → `.gitea/workflows/sync-beta.yml.disabled`
3. Rename `.gitea/workflows/sync-inbound.yml` → `.gitea/workflows/sync-inbound.yml.disabled`
4. Rename `.gitea/workflows/sync-outbound.yml` → `.gitea/workflows/sync-outbound.yml.disabled`
5. Commit with message: `ci: disable release-mirror-github, sync-beta, sync-inbound, sync-outbound workflows [skip ci]`
6. Push to `beta`

### Master Branch
1. Rename `.gitea/workflows/sync-beta.yml` → `.gitea/workflows/sync-beta.yml.disabled`
2. Rename `.gitea/workflows/sync-outbound.yml` → `.gitea/workflows/sync-outbound.yml.disabled`
3. Commit with message: `ci: disable sync-beta, sync-outbound workflows [skip ci]`
4. Push to `master`

## Validation
- Check Gitea Actions UI: no new workflow runs should appear after the commits.
- Existing runs (if any) remain visible in history; only new triggers are suppressed.

## Risks & Mitigations
| Risk | Mitigation |
|------|------------|
| Accidentally triggering builds | Ensure `[skip ci]` is in the commit message; verify before pushing |
| Wrong files renamed | Double-check file list per branch before committing |
| Re-enable needed later | Simply rename `.disabled` back to `.yml` and commit |

## Open Questions
- None. All decisions resolved.

## Notes
- Do NOT use `if: false` job-level guard again—it doesn't prevent Gitea from registering the workflow run.
- Do NOT delete files—renaming preserves history and is reversible.
- Do NOT move files to a different directory—renaming is simpler and achieves the same de-registration.
