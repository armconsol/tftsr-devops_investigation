# LiteLLM + AWS Bedrock Setup

This guide covers how to use **Claude via AWS Bedrock** with TRCAA through the LiteLLM proxy, providing an OpenAI-compatible API gateway.

## Why LiteLLM + Bedrock?

- **Enterprise AWS contracts** — Use existing AWS Bedrock credits instead of direct Anthropic API
- **Multiple AWS accounts** — Run personal and business Bedrock accounts simultaneously
- **OpenAI-compatible API** — Works with any tool expecting OpenAI's API format
- **Claude Code integration** — Reuse the same AWS credentials used by Claude Code CLI

---

## Prerequisites

- **AWS account** with Bedrock access and Claude models enabled
- **AWS credentials** configured (either default profile or named profile)
- **Python 3.8+** for LiteLLM installation

---

## Installation

### 1. Install LiteLLM

```bash
pip install 'litellm[proxy]'
```

Verify installation:
```bash
litellm --version
```

---

## Basic Setup — Single AWS Account

### 1. Create Configuration File

Create `~/.litellm/config.yaml`:

```yaml
model_list:
  - model_name: bedrock-claude
    litellm_params:
      model: bedrock/us.anthropic.claude-sonnet-4-6
      aws_region_name: us-east-1
      # Uses default AWS credentials from ~/.aws/credentials

general_settings:
  master_key: sk-1234  # Any value — used for API authentication
```

### 2. Start LiteLLM Proxy

```bash
# Run in background
nohup litellm --config ~/.litellm/config.yaml --port 8000 > ~/.litellm/litellm.log 2>&1 &

# Verify it's running
ps aux | grep litellm
```

### 3. Test the Connection

```bash
curl http://localhost:8000/v1/chat/completions \
  -H "Authorization: Bearer sk-1234" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "bedrock-claude",
    "messages": [{"role": "user", "content": "Hello"}],
    "max_tokens": 50
  }'
```

Expected response:
```json
{
  "id": "chatcmpl-...",
  "model": "bedrock-claude",
  "choices": [{
    "message": {
      "role": "assistant",
      "content": "Hello! How can I help you today?"
    }
  }]
}
```

### 4. Configure TRCAA

In **Settings → AI Providers → Add Provider**:

| Field | Value |
|-------|-------|
| Provider Name | OpenAI |
| Base URL | `http://localhost:8000/v1` |
| API Key | `sk-1234` (or your master_key from config) |
| Model | `bedrock-claude` |
| Display Name | `Bedrock Claude` |

---

## Advanced Setup — Multiple AWS Accounts

If you have **personal** and **business** Bedrock accounts, you can run both through the same LiteLLM instance.

### 1. Configure AWS Profiles

Ensure you have AWS profiles set up in `~/.aws/credentials`:

```ini
[default]
aws_access_key_id = AKIA...
aws_secret_access_key = ...

[ClaudeCodeLP]
aws_access_key_id = AKIA...
aws_secret_access_key = ...
```

Or if using credential process (like Claude Code does):

```ini
# ~/.aws/config
[profile ClaudeCodeLP]
credential_process = /Users/${USER}/claude-code-with-bedrock/credential-process --profile ClaudeCodeLP
region = us-east-1
```

### 2. Update Configuration File

Edit `~/.litellm/config.yaml`:

```yaml
model_list:
  - model_name: bedrock-personal
    litellm_params:
      model: bedrock/us.anthropic.claude-sonnet-4-6
      aws_region_name: us-east-1
      # Uses default AWS credentials

  - model_name: bedrock-business
    litellm_params:
      model: bedrock/us.anthropic.claude-sonnet-4-5-20250929-v1:0
      aws_region_name: us-east-1
      aws_profile_name: ClaudeCodeLP  # Named profile for business account

general_settings:
  master_key: sk-1234
```

### 3. Restart LiteLLM

```bash
# Find and stop existing process
pkill -f "litellm --config"

# Start with new config
nohup litellm --config ~/.litellm/config.yaml --port 8000 > ~/.litellm/litellm.log 2>&1 &
```

### 4. Verify Both Models

```bash
# List available models
curl -s http://localhost:8000/v1/models \
  -H "Authorization: Bearer sk-1234" | python3 -m json.tool

# Test personal account
curl -s http://localhost:8000/v1/chat/completions \
  -H "Authorization: Bearer sk-1234" \
  -H "Content-Type: application/json" \
  -d '{"model": "bedrock-personal", "messages": [{"role": "user", "content": "test"}]}'

# Test business account
curl -s http://localhost:8000/v1/chat/completions \
  -H "Authorization: Bearer sk-1234" \
  -H "Content-Type: application/json" \
  -d '{"model": "bedrock-business", "messages": [{"role": "user", "content": "test"}]}'
```

### 5. Configure in TRCAA

Add both models as separate providers:

**Provider 1 — Personal:**
- Provider: OpenAI
- Base URL: `http://localhost:8000/v1`
- API Key: `sk-1234`
- Model: `bedrock-personal`
- Display Name: `Bedrock (Personal)`

**Provider 2 — Business:**
- Provider: OpenAI
- Base URL: `http://localhost:8000/v1`
- API Key: `sk-1234`
- Model: `bedrock-business`
- Display Name: `Bedrock (Business)`

---

## Claude Code Integration

If you're using [Claude Code](https://claude.ai/claude-code) with AWS Bedrock, you can reuse the same AWS credentials.

### 1. Check Claude Code Settings

Read your Claude Code configuration:

```bash
cat ~/.claude/settings.json
```

Look for:
- `AWS_PROFILE` environment variable (e.g., `ClaudeCodeLP`)
- `awsAuthRefresh` credential process path
- `AWS_REGION` setting

### 2. Use Same Profile in LiteLLM

In `~/.litellm/config.yaml`, add a model using the same profile:

```yaml
model_list:
  - model_name: claude-code-bedrock
    litellm_params:
      model: bedrock/us.anthropic.claude-sonnet-4-5-20250929-v1:0
      aws_region_name: us-east-1
      aws_profile_name: ClaudeCodeLP  # Same as Claude Code
```

Now both Claude Code and TRCAA use the same Bedrock account without duplicate credential management.

---

## Available Claude Models on Bedrock

| Model ID | Name | Context | Best For |
|----------|------|---------|----------|
| `us.anthropic.claude-sonnet-4-6` | Claude Sonnet 4.6 | 200k tokens | Most tasks, best quality |
| `us.anthropic.claude-sonnet-4-5-20250929-v1:0` | Claude Sonnet 4.5 | 200k tokens | High performance |
| `us.anthropic.claude-opus-4-6` | Claude Opus 4.6 | 200k tokens | Complex reasoning |
| `us.anthropic.claude-haiku-4-5-20251001` | Claude Haiku 4.5 | 200k tokens | Speed + cost optimization |

Check your AWS Bedrock console for available models in your region.

---

## Troubleshooting

### Port Already in Use

If port 8000 is occupied:

```bash
# Find what's using the port
lsof -i :8000

# Use a different port
litellm --config ~/.litellm/config.yaml --port 8080
```

Update the Base URL in TRCAA to match: `http://localhost:8080/v1`

### AWS Credentials Not Found

```bash
# Verify AWS CLI works
aws bedrock list-foundation-models --region us-east-1

# Test specific profile
aws bedrock list-foundation-models --region us-east-1 --profile ClaudeCodeLP
```

If AWS CLI fails, fix credentials first before debugging LiteLLM.

### Model Not Available

Error: `Model not found` or `Access denied`

1. Check [AWS Bedrock Console](https://console.aws.amazon.com/bedrock/) → Model Access
2. Request access to Claude models if not enabled
3. Verify model ID matches exactly (case-sensitive)

### LiteLLM Won't Start

Check logs:
```bash
cat ~/.litellm/litellm.log
```

Common issues:
- Missing Python dependencies: `pip install 'litellm[proxy]' --upgrade`
- YAML syntax error: Validate with `python3 -c "import yaml; yaml.safe_load(open('${HOME}/.litellm/config.yaml'))"`

---

## Auto-Start LiteLLM on Boot

### macOS — LaunchAgent

Create `~/Library/LaunchAgents/com.litellm.proxy.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.litellm.proxy</string>
    <key>ProgramArguments</key>
    <array>
        <string>/opt/homebrew/bin/litellm</string>
        <string>--config</string>
        <string>/Users/${USER}/.litellm/config.yaml</string>
        <string>--port</string>
        <string>8000</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/Users/${USER}/.litellm/litellm.log</string>
    <key>StandardErrorPath</key>
    <string>/Users/${USER}/.litellm/litellm-error.log</string>
</dict>
</plist>
```

Load it:
```bash
launchctl load ~/Library/LaunchAgents/com.litellm.proxy.plist
```

### Linux — systemd

Create `/etc/systemd/system/litellm.service`:

```ini
[Unit]
Description=LiteLLM Proxy
After=network.target

[Service]
Type=simple
User=${USER}
ExecStart=/usr/local/bin/litellm --config /home/${USER}/.litellm/config.yaml --port 8000
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl enable litellm
sudo systemctl start litellm
sudo systemctl status litellm
```

---

## Cost Comparison

| Provider | Model | Input (per 1M tokens) | Output (per 1M tokens) |
|----------|-------|----------------------|----------------------|
| **Anthropic Direct** | Claude Sonnet 4 | $3.00 | $15.00 |
| **AWS Bedrock** | Claude Sonnet 4 | $3.00 | $15.00 |

Pricing is identical, but Bedrock provides:
- AWS consolidated billing
- AWS Credits applicability
- Integration with AWS services (S3, Lambda, etc.)
- Enterprise support contracts

---

## Security Notes

1. **Master Key** — The `master_key` in config is required but doesn't need to be complex since LiteLLM runs locally
2. **AWS Credentials** — Never commit `.aws/credentials` or credential process scripts to git
3. **Local Only** — LiteLLM proxy should only bind to `127.0.0.1` (localhost) — never expose to network
4. **Audit Logs** — TRCAA logs all AI requests with SHA-256 hashes in the audit table

---

## References

- [LiteLLM Documentation](https://docs.litellm.ai/)
- [AWS Bedrock Claude Models](https://docs.aws.amazon.com/bedrock/latest/userguide/models-claude.html)
- [Claude Code on Bedrock](https://docs.anthropic.com/claude/docs/claude-code)
