#!/usr/bin/env sh
# test_reconcile.sh — self-contained test for reconcile.sh using local bare repos
# that stand in for Gitea and GitHub. Verifies:
#   - initial alignment (Gitea content -> GitHub) while preserving each side's .github
#   - outbound-only change (Gitea -> GitHub)
#   - inbound-only change (GitHub -> Gitea)
#   - both-changed conflict (Gitea wins) + non-conflicting GitHub file pulled in
#   - deletion propagation (GitHub deletion -> Gitea)
#   - .github never crosses in either direction
set -eu

HERE="$(cd "$(dirname "$0")" && pwd)"
RECONCILE="${HERE}/reconcile.sh"
ROOT="$(mktemp -d)"
trap 'rm -rf "${ROOT}"' EXIT INT TERM

GITEA="${ROOT}/gitea.git"
GITHUB="${ROOT}/github.git"
FAIL=0

pass() { printf '  ok   %s\n' "$1"; }
fail() { printf '  FAIL %s\n' "$1"; FAIL=$((FAIL+1)); }

# Read a file's content from a bare repo branch; empty if absent.
# Use --git-dir so it works regardless of safe.bareRepository config.
show() { git --git-dir="$1" show "$2:$3" 2>/dev/null || true; }
has()  { git --git-dir="$1" cat-file -e "$2:$3" 2>/dev/null; }

assert_eq() {
  if [ "$2" = "$3" ]; then pass "$1"; else fail "$1 (expected [$3], got [$2])"; fi
}
assert_absent() {
  if has "$1" "$2" "$3"; then fail "$4 (path present)"; else pass "$4"; fi
}

run_reconcile() {
  GITEA_REPO_URL="${GITEA}" GITHUB_REPO_URL="${GITHUB}" PAIRS="master:main" \
    DRY_RUN=0 WORKDIR="${ROOT}/wd" sh "${RECONCILE}" >/dev/null 2>"${ROOT}/err.log" || {
      echo "reconcile failed:"; cat "${ROOT}/err.log"; exit 1; }
  rm -rf "${ROOT}/wd"
}

# --- Seed gitea master and github main ---------------------------------------
git init -q --bare "${GITEA}"
git init -q --bare "${GITHUB}"

SEED="${ROOT}/seed"
git init -q "${SEED}"
git -C "${SEED}" config user.email a@b.c
git -C "${SEED}" config user.name a
mkdir -p "${SEED}/src" "${SEED}/.github/workflows"
printf 'shared-v1\n'      > "${SEED}/src/app.txt"
printf 'common\n'         > "${SEED}/README.md"
printf 'gitea-workflow\n' > "${SEED}/.github/workflows/pr-review.yml"
git -C "${SEED}" add -A
git -C "${SEED}" commit -qm "seed gitea"
git -C "${SEED}" branch -M master
git -C "${SEED}" push -q "${GITEA}" master

# github main: same content files but its OWN .github (the 3 workflows)
GH="${ROOT}/ghseed"
git init -q "${GH}"
git -C "${GH}" config user.email a@b.c
git -C "${GH}" config user.name a
mkdir -p "${GH}/src" "${GH}/.github/workflows"
printf 'shared-v1\n' > "${GH}/src/app.txt"
printf 'common\n'    > "${GH}/README.md"
printf 'github-release\n' > "${GH}/.github/workflows/release.yml"
printf 'github-test\n'    > "${GH}/.github/workflows/test.yml"
git -C "${GH}" add -A
git -C "${GH}" commit -qm "seed github"
git -C "${GH}" branch -M main
git -C "${GH}" push -q "${GITHUB}" main

echo "Test 1: initial align (no state) preserves each side's .github"
run_reconcile
assert_eq   "github keeps release.yml" "$(show "${GITHUB}" main .github/workflows/release.yml)" "github-release"
assert_absent "${GITHUB}" main ".github/workflows/pr-review.yml" "gitea pr-review.yml did NOT cross to github"
assert_eq   "gitea keeps pr-review.yml" "$(show "${GITEA}" master .github/workflows/pr-review.yml)" "gitea-workflow"
assert_absent "${GITEA}" master ".github/workflows/release.yml" "github release.yml did NOT cross to gitea"

echo "Test 2: outbound-only (gitea edits src/app.txt)"
git -C "${SEED}" pull -q "${GITEA}" master 2>/dev/null || true
printf 'shared-v2\n' > "${SEED}/src/app.txt"
git -C "${SEED}" commit -qam "gitea edit"
git -C "${SEED}" push -q "${GITEA}" master
run_reconcile
assert_eq "github received gitea edit" "$(show "${GITHUB}" main src/app.txt)" "shared-v2"

echo "Test 3: inbound-only (github edits README.md)"
git -C "${GH}" pull -q "${GITHUB}" main 2>/dev/null || true
printf 'common-edited-on-github\n' > "${GH}/README.md"
git -C "${GH}" commit -qam "github edit"
git -C "${GH}" push -q "${GITHUB}" main
run_reconcile
assert_eq "gitea received github edit" "$(show "${GITEA}" master README.md)" "common-edited-on-github"

echo "Test 4: both-changed conflict on src/app.txt (gitea wins) + github-only new file pulled"
git -C "${SEED}" pull -q "${GITEA}" master 2>/dev/null || true
git -C "${GH}"   pull -q "${GITHUB}" main 2>/dev/null || true
printf 'gitea-wins\n'  > "${SEED}/src/app.txt"
git -C "${SEED}" commit -qam "gitea conflict edit"
git -C "${SEED}" push -q "${GITEA}" master
printf 'github-loses\n' > "${GH}/src/app.txt"
printf 'from-github\n'  > "${GH}/src/newfile.txt"
git -C "${GH}" add -A
git -C "${GH}" commit -qam "github conflict edit + new file"
git -C "${GH}" push -q "${GITHUB}" main
run_reconcile
assert_eq "conflict: gitea version wins on github" "$(show "${GITHUB}" main src/app.txt)" "gitea-wins"
assert_eq "conflict: gitea keeps its version"      "$(show "${GITEA}" master src/app.txt)" "gitea-wins"
assert_eq "github-only new file pulled to gitea"   "$(show "${GITEA}" master src/newfile.txt)" "from-github"

echo "Test 5: deletion on github propagates to gitea"
git -C "${GH}" pull -q "${GITHUB}" main 2>/dev/null || true
git -C "${GH}" rm -q README.md
git -C "${GH}" commit -qam "github delete README"
git -C "${GH}" push -q "${GITHUB}" main
run_reconcile
assert_absent "${GITEA}" master "README.md" "README.md deleted on gitea too"

echo "Test 6: creating a new GitHub branch (beta) inherits GitHub's .github"
git -C "${SEED}" pull -q "${GITEA}" master 2>/dev/null || true
git -C "${SEED}" checkout -q -B beta
printf 'beta-content\n' > "${SEED}/src/app.txt"
git -C "${SEED}" commit -qam "gitea beta"
git -C "${SEED}" push -q "${GITEA}" beta
GITEA_REPO_URL="${GITEA}" GITHUB_REPO_URL="${GITHUB}" PAIRS="beta:beta" \
  DRY_RUN=0 WORKDIR="${ROOT}/wd6" sh "${RECONCILE}" >/dev/null 2>"${ROOT}/err6.log" || {
    echo "reconcile(beta) failed:"; cat "${ROOT}/err6.log"; exit 1; }
assert_eq     "github beta has gitea content" "$(show "${GITHUB}" beta src/app.txt)" "beta-content"
assert_eq     "github beta inherited release.yml" "$(show "${GITHUB}" beta .github/workflows/release.yml)" "github-release"
assert_absent "${GITHUB}" beta ".github/workflows/pr-review.yml" "gitea pr-review.yml not on github beta"

echo
if [ "${FAIL}" -eq 0 ]; then echo "ALL TESTS PASSED"; else echo "${FAIL} TEST(S) FAILED"; exit 1; fi
