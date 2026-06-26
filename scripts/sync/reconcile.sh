#!/usr/bin/env sh
# reconcile.sh — content-level two-way reconciliation between a Gitea branch and a
# GitHub branch, excluding the `.github/` directory on both sides.
#
# Gitea is authoritative: on a file that changed on BOTH sides since the last
# reconcile, the Gitea version wins. Files changed only on GitHub are pulled into
# Gitea; files changed only on Gitea are pushed to GitHub. Each side keeps its own
# `.github/` directory (GitHub keeps its workflows; Gitea keeps pr-review.yml).
#
# State is persisted as a commit chain on Gitea branches `sync-state-<gitea_branch>`
# whose tree is the last reconciled CONTENT tree (already excludes `.github/`), so
# the base objects survive across fresh CI runners.
#
# All operations are performed with git plumbing on bare tree objects — no working
# tree materialisation, no rsync — which makes the `.github/` carve-out exact.
#
# Required environment:
#   GITEA_REPO_URL    Authenticated push URL for Gitea
#   GITHUB_REPO_URL   Authenticated push URL for GitHub
#   PAIRS             Space list of "gitea_branch:github_branch" (e.g. "master:main beta:beta")
# Optional:
#   SYNC_BOT_NAME     default "gitea-sync-bot"
#   SYNC_BOT_EMAIL    default "sync@tftsr.com"
#   SKIP_MARKER       default "[skip-sync]"
#   DRY_RUN           "1" to compute and log actions without pushing
#   WORKDIR           scratch dir (default: mktemp)
set -eu

SYNC_BOT_NAME="${SYNC_BOT_NAME:-gitea-sync-bot}"
SYNC_BOT_EMAIL="${SYNC_BOT_EMAIL:-sync@tftsr.com}"
SKIP_MARKER="${SKIP_MARKER:-[skip-sync]}"
DRY_RUN="${DRY_RUN:-0}"
EXCLUDE_PATH=".github"

: "${GITEA_REPO_URL:?GITEA_REPO_URL is required}"
: "${GITHUB_REPO_URL:?GITHUB_REPO_URL is required}"
: "${PAIRS:?PAIRS is required (e.g. \"master:main beta:beta\")}"

log() { printf '%s\n' "$*" >&2; }

WORKDIR="${WORKDIR:-$(mktemp -d)}"
REPO="${WORKDIR}/repo"

cleanup() {
  if [ "${KEEP_WORKDIR:-0}" != "1" ] && [ -n "${WORKDIR:-}" ]; then
    rm -rf "${WORKDIR}"
  fi
}
trap cleanup EXIT INT TERM

# ---------------------------------------------------------------------------
# Setup: a single local repo with both remotes fetched.
# ---------------------------------------------------------------------------
setup_repo() {
  rm -rf "${REPO}"
  mkdir -p "${REPO}"
  git -C "${REPO}" init -q
  git -C "${REPO}" config user.name "${SYNC_BOT_NAME}"
  git -C "${REPO}" config user.email "${SYNC_BOT_EMAIL}"
  git -C "${REPO}" remote add gitea "${GITEA_REPO_URL}"
  git -C "${REPO}" remote add github "${GITHUB_REPO_URL}"
  # Fetch all heads from both remotes (branches + sync-state-* live on gitea).
  git -C "${REPO}" fetch -q gitea '+refs/heads/*:refs/remotes/gitea/*'
  git -C "${REPO}" fetch -q github '+refs/heads/*:refs/remotes/github/*'
}

g() { git -C "${REPO}" "$@"; }

# Does a remote-tracking ref exist?
ref_exists() { g rev-parse --verify -q "$1" >/dev/null 2>&1; }

# Content tree (excluding .github) of a commit-ish. Echoes the tree SHA.
content_tree_of() {
  _ci="$1"
  _idx="${WORKDIR}/idx.$$"
  rm -f "${_idx}"
  GIT_INDEX_FILE="${_idx}" g read-tree "${_ci}"
  GIT_INDEX_FILE="${_idx}" g rm -r --cached --quiet --ignore-unmatch "${EXCLUDE_PATH}" >/dev/null 2>&1 || true
  GIT_INDEX_FILE="${_idx}" g write-tree
  rm -f "${_idx}"
}

# Tree SHA of a subdirectory inside a commit-ish (echoes nothing if absent).
subtree_of() {
  g ls-tree "$1" "${EXCLUDE_PATH}" 2>/dev/null | awk '$2=="tree"{print $3}'
}

# Pick an existing GitHub remote branch to inherit .github from when creating a
# brand-new GitHub branch. Prefers main, then master, then the first available.
github_seed_ref() {
  for _b in main master; do
    if ref_exists "refs/remotes/github/${_b}"; then
      echo "refs/remotes/github/${_b}"; return 0
    fi
  done
  g for-each-ref --format='%(refname)' 'refs/remotes/github/*' 2>/dev/null \
    | grep -v '/sync-state-' | head -n1
}

# Paths changed between two trees (name only).
changed_paths() { g diff --name-only "$1" "$2"; }

# Build merged content tree M into a temp index and echo its tree SHA.
#   $1 base tree (B)  $2 gitea content tree (CG)  $3 github content tree (CH)
# Start from CG, then overlay github-only changes (paths changed B->CH but NOT B->CG).
build_merged_tree() {
  _base="$1"; _cg="$2"; _ch="$3"
  _idx="${WORKDIR}/midx.$$"
  rm -f "${_idx}"
  GIT_INDEX_FILE="${_idx}" g read-tree "${_cg}"

  _gitea_changed="$(changed_paths "${_base}" "${_cg}")"
  _github_changed="$(changed_paths "${_base}" "${_ch}")"

  # Iterate github-changed paths; skip those gitea also changed (Gitea wins).
  printf '%s\n' "${_github_changed}" | while IFS= read -r p; do
    [ -n "${p}" ] || continue
    if printf '%s\n' "${_gitea_changed}" | grep -qxF "${p}"; then
      continue   # conflict -> Gitea wins, keep CG version already in index
    fi
    # Determine github's version of this path.
    _entry="$(g ls-tree "${_ch}" -- "${p}")"
    if [ -z "${_entry}" ]; then
      # Deleted on github -> remove from M.
      GIT_INDEX_FILE="${_idx}" g rm --cached --quiet --ignore-unmatch -- "${p}" >/dev/null 2>&1 || true
    else
      _mode="$(printf '%s' "${_entry}" | awk '{print $1}')"
      _sha="$(printf '%s' "${_entry}" | awk '{print $3}')"
      GIT_INDEX_FILE="${_idx}" g update-index --add --cacheinfo "${_mode},${_sha},${p}"
    fi
  done

  GIT_INDEX_FILE="${_idx}" g write-tree
  rm -f "${_idx}"
}

# Compose a full branch tree = content tree M + the target branch's own .github subtree.
#   $1 content tree M   $2 target commit-ish (to source its .github from)
compose_branch_tree() {
  _m="$1"; _target="$2"
  _idx="${WORKDIR}/cidx.$$"
  rm -f "${_idx}"
  GIT_INDEX_FILE="${_idx}" g read-tree "${_m}"
  _gh="$(subtree_of "${_target}")"
  if [ -n "${_gh}" ]; then
    GIT_INDEX_FILE="${_idx}" g read-tree --prefix="${EXCLUDE_PATH}/" "${_gh}"
  fi
  GIT_INDEX_FILE="${_idx}" g write-tree
  rm -f "${_idx}"
}

# Commit a tree onto a branch and push to a remote, if the tree differs.
#   $1 remote  $2 branch  $3 new tree  $4 parent commit-ish (existing branch)  $5 message
commit_and_push() {
  _remote="$1"; _branch="$2"; _tree="$3"; _parent="$4"; _msg="$5"
  _ptree=""
  if [ -n "${_parent}" ] && ref_exists "${_parent}"; then
    _ptree="$(g rev-parse "${_parent}^{tree}")"
  fi
  if [ "${_tree}" = "${_ptree}" ]; then
    log "    = ${_remote}/${_branch}: no change"
    return 0
  fi
  if [ -n "${_parent}" ] && ref_exists "${_parent}"; then
    _commit="$(GIT_AUTHOR_NAME=${SYNC_BOT_NAME} GIT_AUTHOR_EMAIL=${SYNC_BOT_EMAIL} \
      GIT_COMMITTER_NAME=${SYNC_BOT_NAME} GIT_COMMITTER_EMAIL=${SYNC_BOT_EMAIL} \
      g commit-tree "${_tree}" -p "$(g rev-parse "${_parent}")" -m "${_msg}")"
  else
    _commit="$(GIT_AUTHOR_NAME=${SYNC_BOT_NAME} GIT_AUTHOR_EMAIL=${SYNC_BOT_EMAIL} \
      GIT_COMMITTER_NAME=${SYNC_BOT_NAME} GIT_COMMITTER_EMAIL=${SYNC_BOT_EMAIL} \
      g commit-tree "${_tree}" -m "${_msg}")"
  fi
  if [ "${DRY_RUN}" = "1" ]; then
    log "    ~ DRY_RUN would push ${_commit} -> ${_remote}/${_branch}"
    return 0
  fi
  g push -q "${_remote}" "${_commit}:refs/heads/${_branch}"
  log "    > pushed ${_commit} -> ${_remote}/${_branch}"
}

# ---------------------------------------------------------------------------
# Reconcile one pair.
# ---------------------------------------------------------------------------
reconcile_pair() {
  _gb="$1"; _hb="$2"
  _state_branch="sync-state-${_gb}"
  log "── pair gitea:${_gb} ⇄ github:${_hb} ──"

  _gitea_ref="refs/remotes/gitea/${_gb}"
  _github_ref="refs/remotes/github/${_hb}"
  _state_ref="refs/remotes/gitea/${_state_branch}"

  ref_exists "${_gitea_ref}" || { log "  ! gitea/${_gb} missing; skip"; return 0; }

  CG="$(content_tree_of "${_gitea_ref}")"

  if ! ref_exists "${_github_ref}"; then
    # GitHub branch does not exist yet (e.g. creating beta). Seed it from Gitea
    # content, inheriting .github from an existing GitHub branch so the new branch
    # still carries GitHub's workflows.
    log "  github/${_hb} absent -> initializing from gitea content"
    _seed="$(github_seed_ref)"
    if [ -n "${_seed}" ]; then
      _newtree="$(compose_branch_tree "${CG}" "${_seed}")"
    else
      _newtree="${CG}"
    fi
    commit_and_push github "${_hb}" "${_newtree}" "" "chore(sync): initialize ${_hb} from gitea ${_gb} ${SKIP_MARKER}"
    commit_and_push gitea "${_state_branch}" "${CG}" "${_state_ref}" "sync-state: ${_gb} ${SKIP_MARKER}"
    return 0
  fi

  CH="$(content_tree_of "${_github_ref}")"

  if ! ref_exists "${_state_ref}"; then
    # No prior state: treat current Gitea content as authoritative baseline.
    log "  no state -> establishing baseline from gitea; aligning github"
    BR="$(compose_branch_tree "${CG}" "${_github_ref}")"
    commit_and_push github "${_hb}" "${BR}" "${_github_ref}" "chore(sync): align ${_hb} to gitea ${_gb} ${SKIP_MARKER}"
    commit_and_push gitea "${_state_branch}" "${CG}" "" "sync-state: ${_gb} ${SKIP_MARKER}"
    return 0
  fi

  B="$(g rev-parse "${_state_ref}^{tree}")"

  _gitea_changed=0; _github_changed=0
  [ "${CG}" != "${B}" ] && _gitea_changed=1
  [ "${CH}" != "${B}" ] && _github_changed=1

  if [ "${_gitea_changed}" = "0" ] && [ "${_github_changed}" = "0" ]; then
    log "  = both sides unchanged"
    return 0
  fi

  if [ "${_gitea_changed}" = "1" ] && [ "${_github_changed}" = "0" ]; then
    log "  → gitea changed only (push to github)"
    M="${CG}"
  elif [ "${_gitea_changed}" = "0" ] && [ "${_github_changed}" = "1" ]; then
    log "  ← github changed only (pull into gitea)"
    M="${CH}"
  else
    log "  ⇄ both changed (merge; gitea wins conflicts)"
    M="$(build_merged_tree "${B}" "${CG}" "${CH}")"
  fi

  # Apply M to gitea (preserving gitea .github) and github (preserving github .github).
  GTREE="$(compose_branch_tree "${M}" "${_gitea_ref}")"
  HTREE="$(compose_branch_tree "${M}" "${_github_ref}")"
  commit_and_push gitea  "${_gb}" "${GTREE}" "${_gitea_ref}"  "chore(sync): reconcile ${_gb} with github ${_hb} ${SKIP_MARKER}"
  commit_and_push github "${_hb}" "${HTREE}" "${_github_ref}" "chore(sync): reconcile ${_hb} with gitea ${_gb} ${SKIP_MARKER}"
  commit_and_push gitea  "${_state_branch}" "${M}" "${_state_ref}" "sync-state: ${_gb} ${SKIP_MARKER}"
}

main() {
  setup_repo
  for pair in ${PAIRS}; do
    _gb="${pair%%:*}"
    _hb="${pair##*:}"
    reconcile_pair "${_gb}" "${_hb}"
  done
  log "✓ reconcile complete"
}

main "$@"
