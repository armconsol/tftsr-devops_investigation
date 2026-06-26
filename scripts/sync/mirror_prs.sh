#!/usr/bin/env sh
# mirror_prs.sh — mirror OPEN GitHub pull requests into Gitea as review PRs.
#
# For every open PR on the GitHub mirror (any base branch), this:
#   1. pushes the PR head into Gitea as branch `github-pr/<number>`
#   2. opens (or leaves in place) a matching Gitea PR into the mapped base branch
#   3. closes Gitea mirror PRs + deletes their branches once the GitHub PR is gone
#
# Base mapping: GitHub `main` -> Gitea `master`, GitHub `beta` -> Gitea `beta`.
# Any other base falls back to the same-named Gitea branch if it exists, else `beta`.
#
# Required environment:
#   GH_REPO          e.g. msicie/apollo_nxt-trcaa
#   GH_TOKEN         GitHub PAT (GH_SYNC_TOKEN)
#   GITEA_API        e.g. https://gogs.tftsr.com/api/v1/repos/sarman/tftsr-devops_investigation
#   GITEA_TOKEN      Gitea PAT/admin token (can push protected + create PRs)
#   GITEA_PUSH_URL   Authenticated push URL for the Gitea repo
# Optional:
#   PR_PREFIX        default "github-pr"
#   FALLBACK_BASE    default "beta"
#   DRY_RUN          "1" to log without mutating
set -eu

: "${GH_REPO:?GH_REPO required}"
: "${GH_TOKEN:?GH_TOKEN required}"
: "${GITEA_API:?GITEA_API required}"
: "${GITEA_TOKEN:?GITEA_TOKEN required}"
: "${GITEA_PUSH_URL:?GITEA_PUSH_URL required}"
PR_PREFIX="${PR_PREFIX:-github-pr}"
FALLBACK_BASE="${FALLBACK_BASE:-beta}"
DRY_RUN="${DRY_RUN:-0}"

GH_API="https://api.github.com/repos/${GH_REPO}"
WORK="$(mktemp -d)"
trap 'rm -rf "${WORK}"' EXIT INT TERM

log() { printf '%s\n' "$*" >&2; }
gh_get()    { curl -fsSL -H "Authorization: Bearer ${GH_TOKEN}" -H "Accept: application/vnd.github+json" "$@"; }
gitea_get() { curl -fsSL -H "Authorization: token ${GITEA_TOKEN}" "$@"; }
gitea_post(){ curl -fsSL -X POST  -H "Authorization: token ${GITEA_TOKEN}" -H "Content-Type: application/json" "$@"; }
gitea_patch(){ curl -fsSL -X PATCH -H "Authorization: token ${GITEA_TOKEN}" -H "Content-Type: application/json" "$@"; }
gitea_del() { curl -fsS -X DELETE -H "Authorization: token ${GITEA_TOKEN}" "$@"; }

map_base() {
  case "$1" in
    main)   echo master ;;
    beta)   echo beta ;;
    *)
      if gitea_get "${GITEA_API}/branches/$1" >/dev/null 2>&1; then echo "$1"; else echo "${FALLBACK_BASE}"; fi
      ;;
  esac
}

# Local repo for moving PR heads from GitHub into Gitea.
git init -q "${WORK}/repo"
git -C "${WORK}/repo" config user.email "sync@tftsr.com"
git -C "${WORK}/repo" config user.name "gitea-sync-bot"
git -C "${WORK}/repo" remote add github "https://x-access-token:${GH_TOKEN}@github.com/${GH_REPO}.git"
git -C "${WORK}/repo" remote add gitea  "${GITEA_PUSH_URL}"

# --- 1+2: mirror each open GitHub PR ----------------------------------------
PAGE=1
OPEN_NUMBERS=""
while : ; do
  PRS="$(gh_get "${GH_API}/pulls?state=open&per_page=100&page=${PAGE}")"
  COUNT="$(printf '%s' "${PRS}" | jq 'length')"
  [ "${COUNT}" -gt 0 ] || break

  printf '%s' "${PRS}" | jq -c '.[]' | while IFS= read -r pr; do
    NUM="$(printf '%s' "${pr}" | jq -r '.number')"
    TITLE="$(printf '%s' "${pr}" | jq -r '.title')"
    AUTHOR="$(printf '%s' "${pr}" | jq -r '.user.login')"
    BASE="$(printf '%s' "${pr}" | jq -r '.base.ref')"
    URL="$(printf '%s' "${pr}" | jq -r '.html_url')"
    BRANCH="${PR_PREFIX}/${NUM}"
    GBASE="$(map_base "${BASE}")"

    log "── GitHub PR #${NUM} (${BASE} -> gitea:${GBASE}): ${TITLE}"

    # Move PR head into Gitea branch.
    #
    # SECURITY: the PR head is fully attacker-controlled (any external
    # contributor). We must NOT import its CI definitions onto the authoritative
    # Gitea server, or a malicious workflow could run on the Gitea runner with
    # access to repo secrets, and the design's ".github carve-out" would be
    # bypassed. So we rewrite the fetched tree to drop ".github/" and ".gitea/"
    # before it ever lands as a Gitea branch.
    git -C "${WORK}/repo" fetch -q github "pull/${NUM}/head"
    SAFE_TREE="$(git -C "${WORK}/repo" cat-file -p "FETCH_HEAD^{tree}" \
      | awk -F'\t' '$2 != ".github" && $2 != ".gitea"' \
      | git -C "${WORK}/repo" mktree)"
    SAFE_COMMIT="$(printf 'Mirror of GitHub PR #%s (CI definitions stripped) [skip-sync]\n' "${NUM}" \
      | git -C "${WORK}/repo" -c user.name='gitea-sync-bot' -c user.email='sync@tftsr.com' \
          commit-tree "${SAFE_TREE}" -p FETCH_HEAD -F -)"
    if [ "${DRY_RUN}" = "1" ]; then
      log "    ~ DRY_RUN would push sanitized PR head -> gitea/${BRANCH}"
    else
      git -C "${WORK}/repo" push -q -f gitea "${SAFE_COMMIT}:refs/heads/${BRANCH}"
    fi

    # Does an open Gitea PR with this head already exist?
    EXISTING="$(gitea_get "${GITEA_API}/pulls?state=open&limit=100" \
      | jq -r --arg h "${BRANCH}" '[.[] | select(.head.ref == $h)] | length')"

    if [ "${EXISTING}" = "0" ]; then
      BODY="Mirrored from GitHub PR [#${NUM}](${URL}) by @${AUTHOR}.\n\nReview/merge here syncs back to GitHub via the two-way sync."
      PAYLOAD="$(jq -n --arg t "[GitHub #${NUM}] ${TITLE}" --arg b "${BODY}" \
        --arg head "${BRANCH}" --arg base "${GBASE}" \
        '{title:$t, body:$b, head:$head, base:$base}')"
      if [ "${DRY_RUN}" = "1" ]; then
        log "    ~ DRY_RUN would create Gitea PR ${BRANCH} -> ${GBASE}"
      else
        if gitea_post "${GITEA_API}/pulls" -d "${PAYLOAD}" >/dev/null; then
          log "    > created Gitea PR ${BRANCH} -> ${GBASE}"
        else
          log "    ! failed to create Gitea PR for #${NUM}"
        fi
      fi
    else
      log "    = Gitea PR already open for ${BRANCH}"
    fi
  done

  PAGE=$((PAGE+1))
done

# Recompute the set of currently-open GitHub PR numbers for cleanup.
OPEN_NUMBERS="$(gh_get "${GH_API}/pulls?state=open&per_page=100" | jq -r '.[].number' | tr '\n' ' ')"

# --- 3: close Gitea mirror PRs whose GitHub PR is gone -----------------------
gitea_get "${GITEA_API}/pulls?state=open&limit=100" \
  | jq -r --arg p "${PR_PREFIX}/" '.[] | select(.head.ref | startswith($p)) | "\(.number) \(.head.ref)"' \
  | while read -r GP_NUM GP_HEAD; do
      [ -n "${GP_NUM:-}" ] || continue
      SRC_NUM="${GP_HEAD#"${PR_PREFIX}/"}"
      if printf ' %s ' "${OPEN_NUMBERS}" | grep -q " ${SRC_NUM} "; then
        continue   # GitHub PR still open
      fi
      log "── GitHub PR #${SRC_NUM} closed -> closing Gitea PR #${GP_NUM} (${GP_HEAD})"
      if [ "${DRY_RUN}" = "1" ]; then
        log "    ~ DRY_RUN would close Gitea PR #${GP_NUM} and delete ${GP_HEAD}"
      else
        gitea_patch "${GITEA_API}/pulls/${GP_NUM}" -d '{"state":"closed"}' >/dev/null 2>&1 || true
        gitea_del "${GITEA_API}/branches/${GP_HEAD}" >/dev/null 2>&1 || true
      fi
  done

log "✓ PR mirror complete"
