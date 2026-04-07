# Integration Authentication Guide

## Overview

The TRCAA application supports three integration authentication methods, with automatic fallback between them:

1. **API Tokens** (Manual) - Recommended ✅
2. **OAuth 2.0** - Fully automated (when configured)
3. **Browser Cookies** - Partially working ⚠️

## Authentication Priority

When you ask an AI question, the system attempts authentication in this order:

```
1. Extract cookies from persistent browser window
   ↓ (if fails)
2. Use stored API token from database
   ↓ (if fails)
3. Skip that integration and log guidance
```

## HttpOnly Cookie Limitation

**Problem**: Confluence, ServiceNow, and Azure DevOps use **HttpOnly cookies** for security. These cookies:
- ✅ Exist in the persistent browser window
- ✅ Are sent automatically by the browser
- ❌ **Cannot be extracted by JavaScript** (security feature)
- ❌ **Cannot be used in separate HTTP requests**

**Impact**: Cookie extraction via the persistent browser window **fails** for HttpOnly cookies, even though you're logged in.

## Recommended Solution: Use API Tokens

### Confluence Personal Access Token

1. Log into Confluence
2. Go to **Profile → Settings → Personal Access Tokens**
3. Click **Create token**
4. Copy the generated token
5. In TRCAA app:
   - Go to **Settings → Integrations**
   - Find your Confluence integration
   - Click **"Save Manual Token"**
   - Paste the token
   - Token Type: `Bearer`

### ServiceNow API Key

1. Log into ServiceNow
2. Go to **System Security → Application Registry**
3. Click **New → OAuth API endpoint for external clients**
4. Configure and generate API key
5. In TRCAA app:
   - Go to **Settings → Integrations**
   - Find your ServiceNow integration
   - Click **"Save Manual Token"**
   - Paste the API key

### Azure DevOps Personal Access Token (PAT)

1. Log into Azure DevOps
2. Click **User Settings (top right) → Personal Access Tokens**
3. Click **New Token**
4. Scopes: Select **Read** for:
   - Code (for wiki)
   - Work Items (for work item search)
5. Click **Create** and copy the token
6. In TRCAA app:
   - Go to **Settings → Integrations**
   - Find your Azure DevOps integration
   - Click **"Save Manual Token"**
   - Paste the token
   - Token Type: `Bearer`

## Verification

After adding API tokens, test the integration:

1. Open or create an issue
2. Go to Triage page
3. Ask a question like: "How do I upgrade Vesta NXT to 1.0.12"
4. Check the logs for:
   ```
   INFO Using stored cookies for confluence (count: 1)
   INFO Found X integration sources for AI context
   ```

If successful, the AI response should include:
- Content from internal documentation
- Source citations with URLs
- Links to Confluence/ServiceNow/Azure DevOps pages

## Troubleshooting

### No search results found

**Symptom**: AI gives generic answers instead of internal documentation

**Check logs for**:
```
WARN Unable to search confluence - no authentication available
```

**Solution**: Add an API token (see above)

### Cookie extraction timeout

**Symptom**: Logs show:
```
WARN Failed to extract cookies from confluence: Timeout extracting cookies
```

**Why**: HttpOnly cookies cannot be extracted via JavaScript

**Solution**: Use API tokens instead

### Integration not configured

**Symptom**: No integration searches at all

**Check**: Settings → Integrations - ensure integration is added with:
- Base URL configured
- Either browser window open OR API token saved

## Future Enhancements

### Native Cookie Extraction (Planned)

We plan to implement platform-specific native cookie extraction that can access HttpOnly cookies directly from the webview's cookie store:

- **macOS**: Use WKWebView's HTTPCookieStore (requires `cocoa`/`objc` crates)
- **Windows**: Use WebView2's cookie manager (requires `windows` crate)
- **Linux**: Use WebKitGTK cookie manager (requires `webkit2gtk` binding)

This will make the persistent browser approach fully automatic, even with HttpOnly cookies.

### Webview-Based Search (Experimental)

Another approach is to make search requests FROM within the authenticated webview using JavaScript fetch, which automatically includes HttpOnly cookies. This requires reliable IPC communication between JavaScript and Rust.

## Security Notes

### Token Storage

API tokens are:
- ✅ **Encrypted** using AES-256-GCM before storage
- ✅ **Hashed** (SHA-256) for audit logging
- ✅ Stored in encrypted SQLite database
- ✅ Never exposed to frontend JavaScript

### Cookie Storage (when working)

Extracted cookies are:
- ✅ Encrypted before database storage
- ✅ Only retrieved when making API requests
- ✅ Transmitted only over HTTPS

### Audit Trail

All integration authentication attempts are logged:
- Cookie extraction attempts
- Token usage
- Search requests
- Authentication failures

Check **Settings → Security → Audit Log** to review activity.

## Summary

**For reliable integration search NOW**: Use API tokens (Option 1)

**For automatic integration search LATER**: Native cookie extraction will be implemented in a future update

**Current workaround**: API tokens provide full functionality without browser dependency
