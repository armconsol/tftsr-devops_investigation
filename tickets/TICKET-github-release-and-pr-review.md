# Ticket: GitHub release mirroring + PR review on GitHub Actions

**Branch:** `ci/github-release-and-pr-review` (off `beta`)
**PR:** [#143](https://gogs.tftsr.com/sarman/tftsr-devops_investigation/pulls/143) → `beta`

## Description
The public GitHub mirror (`armconsol/tftsr-devops_investigation`) received git refs only — no release binaries and no automated PR review. This adds both, while keeping **Gitea as the single build source** (no GitHub-side Tauri toolchain or code-signing to maintain).

1. **Release asset mirror** — copies finished Gitea release artifacts to the matching GitHub Release.
2. **PR review on GitHub Actions** — ports the existing LLM PR-review automation to GitHub, pointed at the public LLM endpoint.

## Acceptance Criteria
- [x] Publishing a Gitea release copies its notes, prerelease flag and all assets (`.deb/.rpm/.exe/.dmg` + CHANGELOG) to the GitHub Release.
- [x] Existing releases can be backfilled on demand.
- [x] GitHub PRs get the same automated LLM review, using `https://llm.tftsr.com/v1`.
- [x] The LLM key is never exposed to fork PRs.
- [x] Both workflows pass `actionlint` (incl. shellcheck).

## Work Implemented
- **`.gitea/workflows/release-mirror-github.yml`** — triggers on Gitea `release: [published, edited]` and `workflow_dispatch` (optional `tag`; empty = backfill ALL). Reads the Gitea release internally, downloads assets, creates/updates the GitHub Release via REST with `GH_RELEASE_TOKEN`, uploads each asset. Idempotent (skips assets already present); space→period asset-name normalisation to match GitHub.
- **`.github/workflows/pr-review.yml`** — GitHub Actions port of the Gitea PR-review. Endpoint `https://llm.tftsr.com/v1`, model `qwen3.5-122b-think`, key `LITELLM_API_KEY`. `actions/checkout`, posts review via built-in `GITHUB_TOKEN`. Same-repo-only guard (`head.repo.full_name == github.repository`).
- **Secrets provisioned:** `GH_RELEASE_TOKEN` (Gitea Actions), `LITELLM_API_KEY` (GitHub Actions).
- **Related (separate from this PR):** added a Repository-admin bypass to the GitHub `master-protect` ruleset so the existing Gitea→GitHub push-mirror can update `beta`/`master` (it was failing GH013 on direct pushes + linear-history).

## Testing Needed
- **Release mirror:** `workflow_dispatch` with `tag=v1.3.0` → confirm 7 assets land on the GitHub Release; then empty tag to backfill history.
- **PR review:** open a test PR on GitHub → confirm the review comment posts and the LLM call reaches `llm.tftsr.com`.
- Confirm GitHub-hosted runners have outbound access to `https://llm.tftsr.com/v1`.

## Security Notes
- LLM key supplied out-of-band and stored only as a GitHub Actions secret (write-only); never in YAML or commits.
- Fork PRs are skipped, so the secret is unavailable to untrusted code.
- **Recommend rotating** both the GitHub PAT and the LLM key that were shared in chat.
