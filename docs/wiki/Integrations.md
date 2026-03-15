# Integrations

> **Status: All integrations are v0.2 stubs.** They are implemented as placeholder commands that return `"not yet available"` errors. The authentication framework and command signatures are finalized, but the actual API calls are not yet implemented.

---

## Confluence

**Purpose:** Publish RCA and post-mortem documents to a Confluence space.

**Commands:**
- `test_confluence_connection(base_url, credentials)` — Verify credentials
- `publish_to_confluence(doc_id, space_key, parent_page_id?)` — Create/update page

**Planned implementation:**
- Confluence REST API v2: `POST /wiki/rest/api/content`
- Auth: Basic auth (email + API token) or OAuth2
- Page format: Convert Markdown → Confluence storage format (XHTML-like)

**Configuration (Settings → Integrations → Confluence):**
```
Base URL:   https://yourorg.atlassian.net
Email:      user@example.com
API Token:  (stored in Stronghold)
Space Key:  PROJ
```

---

## ServiceNow

**Purpose:** Create incident records in ServiceNow from TFTSR issues.

**Commands:**
- `test_servicenow_connection(instance_url, credentials)` — Verify credentials
- `create_servicenow_incident(issue_id, config)` — Create incident

**Planned implementation:**
- ServiceNow Table API: `POST /api/now/table/incident`
- Auth: Basic auth or OAuth2 bearer token
- Field mapping: TFTSR severity → ServiceNow priority (P1=Critical, P2=High, etc.)

**Configuration:**
```
Instance URL: https://yourorg.service-now.com
Username:     admin
Password:     (stored in Stronghold)
```

---

## Azure DevOps

**Purpose:** Create work items (bugs/incidents) in Azure DevOps from TFTSR issues.

**Commands:**
- `test_azuredevops_connection(org_url, credentials)` — Verify credentials
- `create_azuredevops_workitem(issue_id, project, config)` — Create work item

**Planned implementation:**
- Azure DevOps REST API: `POST /{organization}/{project}/_apis/wit/workitems/${type}`
- Auth: Personal Access Token (PAT) via Basic auth header
- Work item type: Bug or Incident

**Configuration:**
```
Organization URL: https://dev.azure.com/yourorg
Personal Access Token: (stored in Stronghold)
Project:          MyProject
Work Item Type:   Bug
```

---

## v0.2 Roadmap

Integration implementation order (planned):

1. **Confluence** — Most commonly requested; Markdown-to-Confluence conversion library needed
2. **Azure DevOps** — Clean REST API, straightforward PAT auth
3. **ServiceNow** — More complex field mapping; may require customer-specific configuration

Each integration will also require:
- Audit log entry on every publish action
- PII check on document content before external publish
- Connection test UI in Settings → Integrations

---

## Adding an Integration

1. Implement the logic in `src-tauri/src/integrations/{name}.rs`
2. Remove the stub `Err("not yet available")` return in `commands/integrations.rs`
3. Add the new API endpoint to the Tauri CSP `connect-src`
4. Add Stronghold secret key for the API credentials
5. Wire up the Settings UI in `src/pages/Settings/Integrations.tsx`
6. Add audit log call before the external API request
