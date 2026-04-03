import React, { useState } from "react";
import { ExternalLink, Check, X, Loader2, Key, Globe, Lock } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import {
  Card,
  CardHeader,
  CardTitle,
  CardContent,
  CardDescription,
  Button,
  Input,
  Label,
  RadioGroup,
  RadioGroupItem,
} from "@/components/ui";
import {
  initiateOauthCmd,
  authenticateWithWebviewCmd,
  extractCookiesFromWebviewCmd,
  saveManualTokenCmd,
  testConfluenceConnectionCmd,
  testServiceNowConnectionCmd,
  testAzureDevOpsConnectionCmd,
} from "@/lib/tauriCommands";

type AuthMode = "oauth2" | "webview" | "token";

interface IntegrationConfig {
  service: string;
  baseUrl: string;
  username?: string;
  projectName?: string;
  spaceKey?: string;
  connected: boolean;
  authMode: AuthMode;
  token?: string;
  tokenType?: string;
  webviewId?: string;
}

export default function Integrations() {
  const [configs, setConfigs] = useState<Record<string, IntegrationConfig>>({
    confluence: {
      service: "confluence",
      baseUrl: "",
      spaceKey: "",
      connected: false,
      authMode: "webview",
      tokenType: "Bearer",
    },
    servicenow: {
      service: "servicenow",
      baseUrl: "",
      username: "",
      connected: false,
      authMode: "token",
      tokenType: "Basic",
    },
    azuredevops: {
      service: "azuredevops",
      baseUrl: "",
      projectName: "",
      connected: false,
      authMode: "webview",
      tokenType: "Bearer",
    },
  });

  const [loading, setLoading] = useState<Record<string, boolean>>({});
  const [testResults, setTestResults] = useState<Record<string, { success: boolean; message: string } | null>>({});

  const handleAuthModeChange = (service: string, mode: AuthMode) => {
    setConfigs((prev) => ({
      ...prev,
      [service]: { ...prev[service], authMode: mode, connected: false },
    }));
    setTestResults((prev) => ({ ...prev, [service]: null }));
  };

  const handleConnectOAuth = async (service: string) => {
    setLoading((prev) => ({ ...prev, [service]: true }));

    try {
      const response = await initiateOauthCmd(service);

      // Open auth URL in default browser
      await invoke("plugin:shell|open", { path: response.auth_url });

      setConfigs((prev) => ({
        ...prev,
        [service]: { ...prev[service], connected: true },
      }));

      setTestResults((prev) => ({
        ...prev,
        [service]: { success: true, message: "Authentication window opened. Complete the login to continue." },
      }));
    } catch (err) {
      console.error("Failed to initiate OAuth:", err);
      setTestResults((prev) => ({
        ...prev,
        [service]: { success: false, message: String(err) },
      }));
    } finally {
      setLoading((prev) => ({ ...prev, [service]: false }));
    }
  };

  const handleConnectWebview = async (service: string) => {
    const config = configs[service];
    setLoading((prev) => ({ ...prev, [service]: true }));

    try {
      const response = await authenticateWithWebviewCmd(service, config.baseUrl);

      setConfigs((prev) => ({
        ...prev,
        [service]: { ...prev[service], webviewId: response.webview_id },
      }));

      setTestResults((prev) => ({
        ...prev,
        [service]: { success: true, message: response.message + " Click 'Complete Login' when done." },
      }));
    } catch (err) {
      console.error("Failed to open webview:", err);
      setTestResults((prev) => ({
        ...prev,
        [service]: { success: false, message: String(err) },
      }));
    } finally {
      setLoading((prev) => ({ ...prev, [service]: false }));
    }
  };

  const handleCompleteWebviewLogin = async (service: string) => {
    const config = configs[service];
    if (!config.webviewId) {
      setTestResults((prev) => ({
        ...prev,
        [service]: { success: false, message: "No webview session found. Click 'Login via Browser' first." },
      }));
      return;
    }

    setLoading((prev) => ({ ...prev, [`complete-${service}`]: true }));

    try {
      const result = await extractCookiesFromWebviewCmd(service, config.webviewId);

      setConfigs((prev) => ({
        ...prev,
        [service]: { ...prev[service], connected: true, webviewId: undefined },
      }));

      setTestResults((prev) => ({
        ...prev,
        [service]: { success: result.success, message: result.message },
      }));
    } catch (err) {
      console.error("Failed to extract cookies:", err);
      setTestResults((prev) => ({
        ...prev,
        [service]: { success: false, message: String(err) },
      }));
    } finally {
      setLoading((prev) => ({ ...prev, [`complete-${service}`]: false }));
    }
  };

  const handleSaveToken = async (service: string) => {
    const config = configs[service];
    if (!config.token) {
      setTestResults((prev) => ({
        ...prev,
        [service]: { success: false, message: "Please enter a token" },
      }));
      return;
    }

    setLoading((prev) => ({ ...prev, [`save-${service}`]: true }));

    try {
      const result = await saveManualTokenCmd({
        service,
        token: config.token,
        token_type: config.tokenType || "Bearer",
        base_url: config.baseUrl,
      });

      if (result.success) {
        setConfigs((prev) => ({
          ...prev,
          [service]: { ...prev[service], connected: true },
        }));
      }

      setTestResults((prev) => ({
        ...prev,
        [service]: result,
      }));
    } catch (err) {
      console.error("Failed to save token:", err);
      setTestResults((prev) => ({
        ...prev,
        [service]: { success: false, message: String(err) },
      }));
    } finally {
      setLoading((prev) => ({ ...prev, [`save-${service}`]: false }));
    }
  };

  const handleTestConnection = async (service: string) => {
    setLoading((prev) => ({ ...prev, [`test-${service}`]: true }));
    setTestResults((prev) => ({ ...prev, [service]: null }));

    try {
      const config = configs[service];
      let result;

      switch (service) {
        case "confluence":
          result = await testConfluenceConnectionCmd(config.baseUrl, {
            space_key: config.spaceKey,
          });
          break;
        case "servicenow":
          result = await testServiceNowConnectionCmd(config.baseUrl, {
            username: config.username,
          });
          break;
        case "azuredevops":
          result = await testAzureDevOpsConnectionCmd(config.baseUrl, {
            project: config.projectName,
          });
          break;
        default:
          throw new Error(`Unknown service: ${service}`);
      }

      setTestResults((prev) => ({ ...prev, [service]: result }));
    } catch (err) {
      setTestResults((prev) => ({
        ...prev,
        [service]: { success: false, message: String(err) },
      }));
    } finally {
      setLoading((prev) => ({ ...prev, [`test-${service}`]: false }));
    }
  };

  const updateConfig = (service: string, field: string, value: string) => {
    setConfigs((prev) => ({
      ...prev,
      [service]: { ...prev[service], [field]: value },
    }));
  };

  const renderAuthSection = (service: string) => {
    const config = configs[service];
    const isOAuthSupported = service !== "servicenow"; // ServiceNow doesn't support OAuth2

    return (
      <div className="space-y-4">
        {/* Auth Mode Selection */}
        <div className="space-y-3">
          <Label>Authentication Method</Label>
          <RadioGroup
            value={config.authMode}
            onValueChange={(value) => handleAuthModeChange(service, value as AuthMode)}
          >
            {isOAuthSupported && (
              <div className="flex items-center space-x-2">
                <RadioGroupItem value="oauth2" id={`${service}-oauth`} />
                <Label htmlFor={`${service}-oauth`} className="font-normal cursor-pointer flex items-center gap-2">
                  <Lock className="w-4 h-4" />
                  OAuth2 (Enterprise SSO)
                </Label>
              </div>
            )}
            <div className="flex items-center space-x-2">
              <RadioGroupItem value="webview" id={`${service}-webview`} />
              <Label htmlFor={`${service}-webview`} className="font-normal cursor-pointer flex items-center gap-2">
                <Globe className="w-4 h-4" />
                Browser Login (Works off-VPN)
              </Label>
            </div>
            <div className="flex items-center space-x-2">
              <RadioGroupItem value="token" id={`${service}-token`} />
              <Label htmlFor={`${service}-token`} className="font-normal cursor-pointer flex items-center gap-2">
                <Key className="w-4 h-4" />
                Manual Token/API Key
              </Label>
            </div>
          </RadioGroup>
        </div>

        {/* OAuth2 Mode */}
        {config.authMode === "oauth2" && (
          <div className="space-y-3 p-4 bg-muted/30 rounded-lg">
            <p className="text-sm text-muted-foreground">
              OAuth2 requires pre-registered application credentials. This may not work in all enterprise environments.
            </p>
            <Button
              onClick={() => handleConnectOAuth(service)}
              disabled={loading[service] || !config.baseUrl}
            >
              {loading[service] ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Connecting...
                </>
              ) : config.connected ? (
                <>
                  <Check className="w-4 h-4 mr-2" />
                  Connected
                </>
              ) : (
                "Connect with OAuth2"
              )}
            </Button>
          </div>
        )}

        {/* Webview Mode */}
        {config.authMode === "webview" && (
          <div className="space-y-3 p-4 bg-muted/30 rounded-lg">
            <p className="text-sm text-muted-foreground">
              Opens an embedded browser for you to log in normally. Works even when off-VPN. Captures session cookies for API access.
            </p>
            <div className="flex gap-2">
              <Button
                onClick={() => handleConnectWebview(service)}
                disabled={loading[service] || !config.baseUrl}
              >
                {loading[service] ? (
                  <>
                    <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                    Opening...
                  </>
                ) : (
                  "Login via Browser"
                )}
              </Button>
              {config.webviewId && (
                <Button
                  variant="secondary"
                  onClick={() => handleCompleteWebviewLogin(service)}
                  disabled={loading[`complete-${service}`]}
                >
                  {loading[`complete-${service}`] ? (
                    <>
                      <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                      Saving...
                    </>
                  ) : (
                    "Complete Login"
                  )}
                </Button>
              )}
            </div>
          </div>
        )}

        {/* Token Mode */}
        {config.authMode === "token" && (
          <div className="space-y-3 p-4 bg-muted/30 rounded-lg">
            <p className="text-sm text-muted-foreground">
              Enter a Personal Access Token (PAT), API Key, or Bearer token. Most reliable method but requires manual token generation.
            </p>
            <div className="space-y-2">
              <Label htmlFor={`${service}-token-input`}>Token</Label>
              <Input
                id={`${service}-token-input`}
                type="password"
                placeholder={service === "confluence" ? "Bearer token or API key" : "API token or PAT"}
                value={config.token || ""}
                onChange={(e) => updateConfig(service, "token", e.target.value)}
              />
              <p className="text-xs text-muted-foreground">
                {service === "confluence" && "Generate at: https://id.atlassian.com/manage-profile/security/api-tokens"}
                {service === "azuredevops" && "Generate at: https://dev.azure.com/{org}/_usersSettings/tokens"}
                {service === "servicenow" && "Use your ServiceNow password or API key"}
              </p>
            </div>
            <Button
              onClick={() => handleSaveToken(service)}
              disabled={loading[`save-${service}`] || !config.token}
            >
              {loading[`save-${service}`] ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Validating...
                </>
              ) : (
                "Save & Validate Token"
              )}
            </Button>
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Integrations</h1>
        <p className="text-muted-foreground mt-1">
          Connect TFTSR with your existing tools and platforms. Choose the authentication method that works best for your environment.
        </p>
      </div>

      {/* Confluence */}
      <Card>
        <CardHeader>
          <CardTitle className="text-xl flex items-center gap-2">
            <ExternalLink className="w-5 h-5" />
            Confluence
          </CardTitle>
          <CardDescription>
            Publish RCA documents to Confluence spaces. Supports OAuth2, browser login, or API tokens.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="confluence-url">Base URL</Label>
            <Input
              id="confluence-url"
              placeholder="https://your-domain.atlassian.net"
              value={configs.confluence.baseUrl}
              onChange={(e) => updateConfig("confluence", "baseUrl", e.target.value)}
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="confluence-space">Default Space Key</Label>
            <Input
              id="confluence-space"
              placeholder="DEV"
              value={configs.confluence.spaceKey || ""}
              onChange={(e) => updateConfig("confluence", "spaceKey", e.target.value)}
            />
          </div>

          {renderAuthSection("confluence")}

          <div className="flex items-center gap-3 pt-2">
            <Button
              variant="outline"
              onClick={() => handleTestConnection("confluence")}
              disabled={loading["test-confluence"] || !configs.confluence.connected}
            >
              {loading["test-confluence"] ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Testing...
                </>
              ) : (
                "Test Connection"
              )}
            </Button>
          </div>

          {testResults.confluence && (
            <div
              className={`p-3 rounded text-sm ${
                testResults.confluence.success
                  ? "bg-green-500/10 text-green-700 dark:text-green-400"
                  : "bg-destructive/10 text-destructive"
              }`}
            >
              {testResults.confluence.success ? (
                <Check className="w-4 h-4 inline mr-2" />
              ) : (
                <X className="w-4 h-4 inline mr-2" />
              )}
              {testResults.confluence.message}
            </div>
          )}
        </CardContent>
      </Card>

      {/* ServiceNow */}
      <Card>
        <CardHeader>
          <CardTitle className="text-xl flex items-center gap-2">
            <ExternalLink className="w-5 h-5" />
            ServiceNow
          </CardTitle>
          <CardDescription>
            Link incidents and push resolution steps. Supports browser login or basic authentication.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="servicenow-url">Instance URL</Label>
            <Input
              id="servicenow-url"
              placeholder="https://your-instance.service-now.com"
              value={configs.servicenow.baseUrl}
              onChange={(e) => updateConfig("servicenow", "baseUrl", e.target.value)}
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="servicenow-username">Username</Label>
            <Input
              id="servicenow-username"
              placeholder="admin"
              value={configs.servicenow.username || ""}
              onChange={(e) => updateConfig("servicenow", "username", e.target.value)}
            />
          </div>

          {renderAuthSection("servicenow")}

          <div className="flex items-center gap-3 pt-2">
            <Button
              variant="outline"
              onClick={() => handleTestConnection("servicenow")}
              disabled={loading["test-servicenow"]}
            >
              {loading["test-servicenow"] ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Testing...
                </>
              ) : (
                "Test Connection"
              )}
            </Button>
          </div>

          {testResults.servicenow && (
            <div
              className={`p-3 rounded text-sm ${
                testResults.servicenow.success
                  ? "bg-green-500/10 text-green-700 dark:text-green-400"
                  : "bg-destructive/10 text-destructive"
              }`}
            >
              {testResults.servicenow.success ? (
                <Check className="w-4 h-4 inline mr-2" />
              ) : (
                <X className="w-4 h-4 inline mr-2" />
              )}
              {testResults.servicenow.message}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Azure DevOps */}
      <Card>
        <CardHeader>
          <CardTitle className="text-xl flex items-center gap-2">
            <ExternalLink className="w-5 h-5" />
            Azure DevOps
          </CardTitle>
          <CardDescription>
            Create work items and attach RCA documents. Supports OAuth2, browser login, or PAT tokens.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="ado-url">Organization URL</Label>
            <Input
              id="ado-url"
              placeholder="https://dev.azure.com/your-org"
              value={configs.azuredevops.baseUrl}
              onChange={(e) => updateConfig("azuredevops", "baseUrl", e.target.value)}
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="ado-project">Default Project</Label>
            <Input
              id="ado-project"
              placeholder="MyProject"
              value={configs.azuredevops.projectName || ""}
              onChange={(e) => updateConfig("azuredevops", "projectName", e.target.value)}
            />
          </div>

          {renderAuthSection("azuredevops")}

          <div className="flex items-center gap-3 pt-2">
            <Button
              variant="outline"
              onClick={() => handleTestConnection("azuredevops")}
              disabled={loading["test-azuredevops"] || !configs.azuredevops.connected}
            >
              {loading["test-azuredevops"] ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Testing...
                </>
              ) : (
                "Test Connection"
              )}
            </Button>
          </div>

          {testResults.azuredevops && (
            <div
              className={`p-3 rounded text-sm ${
                testResults.azuredevops.success
                  ? "bg-green-500/10 text-green-700 dark:text-green-400"
                  : "bg-destructive/10 text-destructive"
              }`}
            >
              {testResults.azuredevops.success ? (
                <Check className="w-4 h-4 inline mr-2" />
              ) : (
                <X className="w-4 h-4 inline mr-2" />
              )}
              {testResults.azuredevops.message}
            </div>
          )}
        </CardContent>
      </Card>

      <div className="p-4 bg-muted/50 rounded-lg space-y-2">
        <p className="text-sm font-semibold">Authentication Method Comparison:</p>
        <ul className="text-xs text-muted-foreground space-y-1 list-disc list-inside">
          <li><strong>OAuth2:</strong> Most secure, but requires pre-registered app. May not work with enterprise SSO.</li>
          <li><strong>Browser Login:</strong> Best for VPN environments. Lets you authenticate off-VPN, extracts session cookies for API use.</li>
          <li><strong>Manual Token:</strong> Most reliable fallback. Requires generating API tokens manually from each service.</li>
        </ul>
      </div>
    </div>
  );
}
