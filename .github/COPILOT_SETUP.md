# GitHub Copilot Code Review Setup

## Overview

GitHub Copilot can automatically review pull requests when properly configured. This document explains how to enable Copilot code reviews for this repository.

## Current Status

✅ **Workflows Active**: GitHub shows Copilot workflows are active:
- `Copilot` (pull-request-reviewer)
- `Copilot cloud agent` (copilot-swe-agent)
- `CodeQL` (code scanning)

⚠️ **Configuration Needed**: Copilot code reviews must be enabled through GitHub Advanced Security settings.

## How GitHub Copilot Code Reviews Work

GitHub Copilot code reviews are **not** triggered via CODEOWNERS file (unlike human reviewers). Instead, they are configured through:

1. **GitHub Advanced Security** (requires GitHub Enterprise or GitHub Team plan)
2. **Repository Settings** → **Security** → **Code security and analysis**
3. **Copilot Autofix** (for security vulnerabilities)
4. **Copilot Code Review** (manual opt-in feature)

## Setup Steps

### Step 1: Enable GitHub Advanced Security

1. Navigate to: `https://github.com/tftsr/apollo_nxt-trcaa/settings/security_analysis`
2. Enable **GitHub Advanced Security** (if available with your plan)
3. Enable **Dependabot alerts**
4. Enable **Code scanning** (CodeQL)
5. Enable **Secret scanning**

### Step 2: Enable Copilot Code Review

As of 2024-2026, GitHub Copilot code reviews can be enabled via:

**Option A: Copilot Autofix (Security-focused)**
1. Go to repository **Settings** → **Code security and analysis**
2. Enable **Copilot Autofix** under "Code scanning"
3. Copilot will suggest fixes for CodeQL alerts in pull requests

**Option B: Copilot Workspace (Preview Feature)**
1. Ensure your organization has Copilot Business or Enterprise
2. Navigate to: `https://github.com/tftsr/apollo_nxt-trcaa/settings/copilot`
3. Enable **Copilot Code Review** (if available)
4. Configure review triggers:
   - On all pull requests
   - On pull requests targeting protected branches
   - Manual trigger only

### Step 3: Configure Review Rules

Add Copilot as a required check in branch protection:

```bash
# Via GitHub CLI
gh api repos/tftsr/apollo_nxt-trcaa/branches/main/protection/required_status_checks \
  --method PATCH \
  --field strict=true \
  --field contexts[]='rust-test' \
  --field contexts[]='frontend-test' \
  --field contexts[]='copilot-code-review'  # Add this line
```

Or via GitHub UI:
1. Go to **Settings** → **Branches** → **Branch protection rules** → **main**
2. Under "Require status checks to pass before merging"
3. Add **copilot-code-review** to required checks

## Verification

To verify Copilot is reviewing PRs:

```bash
# Check if Copilot workflow ran on a PR
gh pr checks 27

# Check for Copilot comments on a PR
gh pr view 27 --comments | grep -i copilot
```

## Triggering Manual Review

If Copilot code review is enabled but not automatic, you can trigger it manually:

1. Add a comment to the PR: `@github-copilot review`
2. Or use GitHub CLI: `gh pr review 27 --request-changes --body "@github-copilot please review"`

## Current Configuration

**Branch Protection** (as of 2026-06-02):
- ✅ Required status checks: `rust-test`, `frontend-test`
- ✅ Require code owner reviews: Yes
- ✅ Required approving review count: 1
- ⚠️ Copilot code review: Not configured as required check

**CODEOWNERS**:
- Owner: @sarman
- Note: `@github-copilot` removed from CODEOWNERS (not a valid reviewer)

## Limitations

- **Plan Requirement**: GitHub Advanced Security requires GitHub Enterprise or Team plan
- **Private Repos**: May have limited Copilot features depending on plan
- **Availability**: Copilot code review features are gradually rolling out
- **Manual Trigger**: Some orgs require manual trigger via comments

## Alternative: CodeQL Analysis

If Copilot code review is not available, CodeQL provides automated code analysis:

1. CodeQL workflow is already active (`.github/workflows/codeql-analysis.yml` - dynamic)
2. Runs on every push to main and pull request
3. Scans for security vulnerabilities and code quality issues
4. Results appear in **Security** → **Code scanning alerts**

## References

- [GitHub Advanced Security Documentation](https://docs.github.com/en/get-started/learning-about-github/about-github-advanced-security)
- [GitHub Copilot for Business](https://docs.github.com/en/copilot/github-copilot-enterprise/overview/about-github-copilot-enterprise)
- [CodeQL Documentation](https://codeql.github.com/)

## Action Items

To fully enable Copilot code reviews on this repo:

1. [ ] Verify GitHub plan includes Advanced Security features
2. [ ] Enable GitHub Advanced Security in repo settings
3. [ ] Enable Copilot Autofix (if available)
4. [ ] Configure Copilot code review triggers (if feature is available)
5. [ ] Add `copilot-code-review` to required status checks
6. [ ] Test on a sample PR to verify functionality

## Contact

For questions about GitHub Advanced Security or Copilot features for the TFTSR organization, contact:
- GitHub Organization Admins
- DevOps Team

---

**Last Updated**: 2026-06-02
**Status**: Configuration pending - awaiting Advanced Security setup
