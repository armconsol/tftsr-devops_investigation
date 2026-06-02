# Azure Boards + GitHub Integration

## Issue

When using `AB#727547` syntax in PR titles or commit messages, the work item reference is **not** automatically converted to a clickable link to Azure DevOps.

## Root Cause

The `AB#` syntax requires the **Azure Boards GitHub App** to be installed and configured for this repository.

## Current Status

❌ **Azure Boards app not installed** on `msicie/apollo_nxt-trcaa`
- `AB#` references in titles/commits are not linked
- Manual URL links work: `https://dev.azure.com/msi-cie/Apollo/_workitems/edit/727547`

## How Azure Boards + GitHub Integration Works

When properly configured:
1. `AB#727547` in PR title → Automatically converted to clickable link
2. `AB#727547` in commit message → Linked to work item
3. PR/commit status → Appears in ADO work item "Development" tab
4. PR merge → Can auto-transition work item state

## Setup Instructions

### Step 1: Install Azure Boards GitHub App

**Option A: Organization-Level Installation** (Recommended)
1. Go to: https://github.com/marketplace/azure-boards
2. Click **"Set up a plan"** or **"Install it for free"**
3. Select **msi-cie** organization
4. Choose **"All repositories"** or select specific repos
5. Click **"Install"**

**Option B: Repository-Level Installation**
1. Go to: https://github.com/apps/azure-boards
2. Click **"Configure"**
3. Select **msi-cie** organization
4. Under "Repository access", select **"Only select repositories"**
5. Choose **apollo_nxt-trcaa**
6. Click **"Save"**

### Step 2: Connect to Azure DevOps

1. After installation, you'll be redirected to Azure DevOps
2. Sign in with your MSI account: `VFK387@motorolasolutions.com`
3. Select **Azure DevOps organization**: `dev.azure.com/msi-cie`
4. Select **Project**: `Apollo`
5. Authorize the connection

### Step 3: Configure Repository Mapping

1. In Azure DevOps, go to: `https://dev.azure.com/msi-cie/Apollo/_settings/boards-external-integration`
2. Click **"+ Add connection"**
3. Select **GitHub** as the source
4. Choose the repository: **msicie/apollo_nxt-trcaa**
5. Configure settings:
   - ✅ Enable **automatic work item linking**
   - ✅ Enable **state transition on PR merge**
   - ✅ Enable **mentions validation**

### Step 4: Verify Integration

After setup, test the integration:

```bash
# Create a test branch
git checkout -b test/azure-boards-link

# Create a commit with AB# reference
git commit --allow-empty -m "test: verify Azure Boards linking AB#727547"

# Push and create PR
git push -u origin test/azure-boards-link
gh pr create --title "Test: Azure Boards Integration AB#727547" --body "Testing AB# linking"
```

Expected results:
- ✅ `AB#727547` in PR title is a clickable link
- ✅ PR appears in ADO work item 727547 "Development" tab
- ✅ Commit with `AB#` appears in work item history

## Available Syntax

Once installed, these formats work:

### In PR Titles and Descriptions
```
AB#727547                    # Basic link
Fixes AB#727547              # Closes work item on merge
Resolves AB#727547           # Closes work item on merge
Closes AB#727547             # Closes work item on merge
```

### In Commit Messages
```
git commit -m "feat: add feature AB#727547"
git commit -m "fix: resolve bug (fixes AB#727547)"
```

### Multiple Work Items
```
feat: implement features AB#727547 AB#744142
```

## State Transitions

Configure automatic state transitions on PR events:

| GitHub Event | ADO Work Item State Transition |
|--------------|--------------------------------|
| PR created with `AB#` | No change (or → Active) |
| PR merged with `Fixes AB#` | → Resolved or Closed |
| PR merged with `AB#` | No change (configurable) |
| PR closed without merge | No change |

## Current Workaround

Until Azure Boards app is installed, use full URLs:

**In PR Description** (already done in PR #27):
```markdown
**Work Item**: https://dev.azure.com/msi-cie/Apollo/_workitems/edit/727547
```

**In Commits**:
```bash
git commit -m "feat: add feature

Work Item: https://dev.azure.com/msi-cie/Apollo/_workitems/edit/727547"
```

## Benefits of Azure Boards Integration

### For Developers
- ✅ Quick navigation from PR to work item
- ✅ See all PRs/commits linked to a work item
- ✅ Automatic work item state updates
- ✅ Reduced manual ADO updates

### For Project Management
- ✅ Visibility into code changes per work item
- ✅ Traceability from requirement → code → deployment
- ✅ Automated status updates
- ✅ Better sprint velocity tracking

### For Compliance
- ✅ Audit trail of code changes per work item
- ✅ Traceability for security/compliance requirements
- ✅ Automated documentation of development activity

## Verification Commands

After installation, verify with:

```bash
# Check if Azure Boards app is installed
gh api repos/msicie/apollo_nxt-trcaa/installation

# View PR with AB# reference
gh pr view 27

# Check work item in ADO for linked PRs
az boards work-item show --id 727547 --org https://dev.azure.com/msi-cie | jq '.relations'
```

## Troubleshooting

### AB# Not Linking
**Problem**: `AB#727547` shows as plain text, not a link

**Solutions**:
1. Verify Azure Boards app is installed for the repo
2. Check Azure DevOps connection is active
3. Ensure repo is mapped in ADO project settings
4. Verify `AB#` format is correct (no spaces)

### PRs Not Appearing in ADO
**Problem**: PR created but doesn't show in work item "Development" tab

**Solutions**:
1. Check if `AB#` was in PR title or description
2. Verify ADO project connection is active
3. Wait 5-10 minutes for sync (can be delayed)
4. Manually link PR in ADO if needed

### State Transitions Not Working
**Problem**: PR merged but work item state unchanged

**Solutions**:
1. Verify state transition rules are configured in ADO
2. Check if `Fixes AB#` syntax was used (not just `AB#`)
3. Ensure PR was merged (not closed without merge)
4. Check ADO project settings for transition rules

## Security Considerations

- Azure Boards app requires **read/write** access to repos
- OAuth token is stored in Azure DevOps
- App can read PR content and commit messages
- All activity is logged in both GitHub and ADO audit logs

## References

- [Azure Boards GitHub App](https://github.com/marketplace/azure-boards)
- [Azure Boards + GitHub Integration Docs](https://learn.microsoft.com/en-us/azure/devops/boards/github/)
- [Work Item Linking Syntax](https://learn.microsoft.com/en-us/azure/devops/boards/github/link-to-from-github)

## Action Items

To enable `AB#` linking on this repo:

1. [ ] Install Azure Boards GitHub app on msi-cie organization or apollo_nxt-trcaa repo
2. [ ] Connect to Azure DevOps (dev.azure.com/msi-cie)
3. [ ] Map repository in Apollo project settings
4. [ ] Configure state transition rules (optional)
5. [ ] Test with a sample PR using `AB#` syntax
6. [ ] Update team documentation with `AB#` syntax usage

## Contact

For questions about Azure Boards integration or GitHub app installation:
- GitHub Organization Admins: @msi-cie admins
- Azure DevOps Project Admins: Apollo project leads
- DevOps Team

---

**Last Updated**: 2026-06-02
**Status**: Azure Boards app not installed - manual URL links required
**Repository**: msicie/apollo_nxt-trcaa
**ADO Organization**: dev.azure.com/msi-cie
**ADO Project**: Apollo
