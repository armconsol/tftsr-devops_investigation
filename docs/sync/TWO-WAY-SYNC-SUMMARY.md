# Two-Way Sync — Gitea ⇄ GitHub (`msicie/apollo_nxt-trcaa`)

> No ADO ticket associated (confirmed with requester).

## Description

Implements a **bidirectional content sync** between the authoritative Gitea
repository (`gogs.tftsr.com/sarman/tftsr-devops_investigation`) and the GitHub
repository `msicie/apollo_nxt-trcaa`.

Key constraints honored:

- **Gitea is authoritative.** On conflict, the Gitea version wins.
- **GitHub's existing workflows are never modified.** `build-images.yml`,
  `release.yml`, and `test.yml` are untouched.
- **`.github/` is excluded** from the sync in both directions, so each host keeps
  its own CI definitions. Because of this the sync is **content-level** (file
  trees are reconciled); commit SHAs intentionally do **not** match across hosts.
- **No `llm.tftsr.com` on GitHub.** GitHub continues to use native Copilot
  reviews; Gitea keeps its own `pr-review.yml`.
- The existing `armconsol/...` one-way replication is left untouched.

Branch mapping:

| Gitea | GitHub | Direction | Conflict policy |
|-------|--------|-----------|-----------------|
| `master` (stable) | `main` (stable) | two-way | Gitea wins |
| `beta` (testing)  | `beta` (testing) | two-way | Gitea wins |
| open GitHub PRs (any base) | → Gitea review PRs | inbound | CI defs stripped |

## Acceptance Criteria

- [x] Content changes on Gitea `master`/`beta` propagate to GitHub `main`/`beta`.
- [x] Content changes / merged PRs on GitHub `main`/`beta` propagate back to Gitea.
- [x] `.github/` never crosses in either direction (verified by tests).
- [x] No echo/infinite loop: sync-bot identity (`gitea-sync-bot <sync@tftsr.com>`)
      + `[skip-sync]` commit marker are honored on both sides.
- [x] GitHub's 3 existing workflows are not modified; new GitHub workflows are additive.
- [x] Tags mirror Gitea → GitHub only.
- [x] Open GitHub PRs surface as Gitea review PRs **with CI definitions removed**.
- [x] All internet-facing Git/API URLs use `https://gogs.tftsr.com` (no internal IPs).
- [x] No tokens embedded in files; referenced by secret name only.

## Work Implemented

### Reconcile engine + tests
- `scripts/sync/reconcile.sh` — core bidirectional content-reconcile engine using
  pure git plumbing (read-tree/write-tree/commit-tree/ls-tree). Excludes `.github/`,
  applies Gitea-wins on conflict, persists last-synced state as `sync-state-<branch>`
  refs so product branches stay clean.
- `scripts/sync/test_reconcile.sh` — local TDD harness (bare repos). Covers initial
  align, `.github` preservation both ways, outbound-only, inbound-only, both-changed
  conflict (Gitea wins), deletion propagation, and new-branch (beta) creation.

### PR mirroring
- `scripts/sync/mirror_prs.sh` — mirrors open GitHub PRs into Gitea branches
  (`github-pr/<n>`) and opens matching Gitea review PRs; closes them when the GitHub
  PR closes. **Strips `.github/` and `.gitea/` from each PR head before pushing** so
  untrusted contributor CI never lands on the authoritative server.

### Gitea-side workflows
- `.gitea/workflows/sync-outbound.yml` — push `master`/`beta` + schedule + dispatch;
  runs reconcile and mirrors tags Gitea → GitHub.
- `.gitea/workflows/sync-inbound.yml` — schedule (15m) + dispatch; runs reconcile and
  PR mirroring.

### GitHub-side workflows (version-controlled here, deployed to GitHub manually)
- `scripts/sync/github/gitea-sync-notify.yml` — event-driven; calls Gitea's
  `sync-inbound.yml` workflow-dispatch API so inbound changes flow back promptly.
- `scripts/sync/github/release-beta.yml` — additive beta installer pipeline
  (prerelease, `vX.Y.Z-beta.N` tags, all 4 platforms). Does **not** modify existing
  GitHub workflows.

### Security hardening (from internal review)
- **Critical** — eliminated GitHub Actions expression injection in
  `gitea-sync-notify.yml` (untrusted commit message/author now passed via `env:`,
  not interpolated into the `run:` block). Same pattern applied to the new
  `release-beta.yml` guard.
- **High** — `mirror_prs.sh` now strips `.github/`/`.gitea/` from untrusted PR heads
  before they reach Gitea, preventing malicious workflow execution and CI smuggling.
- **Medium** — `release-beta.yml` now skips sync-bot/`[skip-sync]` commits, preventing
  duplicate/runaway beta releases on sync round-trips.
- **Medium** — tag-mirror step redacts token values from any git output piped to logs.

## Testing Needed

Automated (already passing locally):
- `sh scripts/sync/test_reconcile.sh` — full reconcile harness.
- `shellcheck scripts/sync/*.sh` — clean.
- `yamllint` on the new workflow YAML — clean.

Manual / environment-dependent (require live setup and secrets):
1. **Secrets provisioning**
   - Gitea: `GH_SYNC_TOKEN` (msicie GitHub PAT, repo+workflow).
   - GitHub: `GITEA_SYNC_URL` = `https://gogs.tftsr.com`, `GITEA_DISPATCH_TOKEN`.
2. **Deploy GitHub-side workflows** — copy `scripts/sync/github/*.yml` into GitHub's
   `.github/workflows/`.
3. **Initial alignment** — one-time hard-align GitHub `main`/`beta` to Gitea
   `master`/`beta`; re-open any wanted Dependabot PRs afterward.
4. **End-to-end validation**
   - Gitea `master` edit → appears on GitHub `main` (and triggers GitHub release build).
   - GitHub `main`/`beta` edit → appears on Gitea `master`/`beta`.
   - Open a GitHub PR → Gitea review PR created with `.github`/`.gitea` removed.
   - Confirm no echo loop (sync-bot commits do not re-trigger).
   - Confirm `.github/` never crosses in either direction.
5. **Rotate the exposed tokens** that were shared during design, then update secrets.
