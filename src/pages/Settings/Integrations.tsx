import React, { useState } from "react";
import { ExternalLink, Check, X, Loader2 } from "lucide-react";
import {
  Card,
  CardHeader,
  CardTitle,
  CardContent,
  CardDescription,
  Button,
  Input,
  Label,
} from "@/components/ui";
import {
  initiateOauthCmd,
  testConfluenceConnectionCmd,
  testServiceNowConnectionCmd,
  testAzureDevOpsConnectionCmd,
} from "@/lib/tauriCommands";
import { invoke } from "@tauri-apps/api/core";

interface IntegrationConfig {
  service: string;
  baseUrl: string;
  username?: string;
  projectName?: string;
  spaceKey?: string;
  connected: boolean;
}

export default function Integrations() {
  const [configs, setConfigs] = useState<Record<string, IntegrationConfig>>({
    confluence: {
      service: "confluence",
      baseUrl: "",
      spaceKey: "",
      connected: false,
    },
    servicenow: {
      service: "servicenow",
      baseUrl: "",
      username: "",
      connected: false,
    },
    azuredevops: {
      service: "azuredevops",
      baseUrl: "",
      projectName: "",
      connected: false,
    },
  });

  const [loading, setLoading] = useState<Record<string, boolean>>({});
  const [testResults, setTestResults] = useState<Record<string, { success: boolean; message: string } | null>>({});

  const handleConnect = async (service: string) => {
    setLoading((prev) => ({ ...prev, [service]: true }));

    try {
      const response = await initiateOauthCmd(service);

      // Open auth URL in default browser using shell plugin
      await invoke("plugin:shell|open", { path: response.auth_url });

      // Mark as connected (optimistic)
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

  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Integrations</h1>
        <p className="text-muted-foreground mt-1">
          Connect TFTSR with your existing tools and platforms via OAuth2.
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
            Publish RCA documents to Confluence spaces. Requires OAuth2 authentication with Atlassian.
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

          <div className="flex items-center gap-3">
            <Button
              onClick={() => handleConnect("confluence")}
              disabled={loading.confluence || !configs.confluence.baseUrl}
            >
              {loading.confluence ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Connecting...
                </>
              ) : configs.confluence.connected ? (
                <>
                  <Check className="w-4 h-4 mr-2" />
                  Connected
                </>
              ) : (
                "Connect with OAuth2"
              )}
            </Button>

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
            Link incidents and push resolution steps. Uses basic authentication (username + password).
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

          <div className="space-y-2">
            <Label htmlFor="servicenow-password">Password</Label>
            <Input
              id="servicenow-password"
              type="password"
              placeholder="••••••••"
              disabled
            />
            <p className="text-xs text-muted-foreground">
              ServiceNow credentials are stored securely after first login. OAuth2 not supported.
            </p>
          </div>

          <div className="flex items-center gap-3">
            <Button
              onClick={() =>
                setTestResults((prev) => ({
                  ...prev,
                  servicenow: {
                    success: false,
                    message: "ServiceNow uses basic authentication, not OAuth2. Enter credentials above.",
                  },
                }))
              }
              disabled={!configs.servicenow.baseUrl || !configs.servicenow.username}
            >
              Save Credentials
            </Button>

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
            Create work items and attach RCA documents. Requires OAuth2 authentication with Microsoft.
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

          <div className="flex items-center gap-3">
            <Button
              onClick={() => handleConnect("azuredevops")}
              disabled={loading.azuredevops || !configs.azuredevops.baseUrl}
            >
              {loading.azuredevops ? (
                <>
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                  Connecting...
                </>
              ) : configs.azuredevops.connected ? (
                <>
                  <Check className="w-4 h-4 mr-2" />
                  Connected
                </>
              ) : (
                "Connect with OAuth2"
              )}
            </Button>

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
        <p className="text-sm font-semibold">How OAuth2 Authentication Works:</p>
        <ol className="text-xs text-muted-foreground space-y-1 list-decimal list-inside">
          <li>Click "Connect with OAuth2" to open the service's authentication page</li>
          <li>Log in with your service credentials in your default browser</li>
          <li>Authorize TFTSR to access your account</li>
          <li>You'll be automatically redirected back and the connection will be saved</li>
          <li>Tokens are encrypted and stored locally in your secure database</li>
        </ol>
      </div>
    </div>
  );
}
