import React, { useState, useEffect, useCallback } from "react";
import { Plus, Pencil, Trash2, RefreshCw, CheckCircle, XCircle, Clock, Plug } from "lucide-react";
import {
  Card,
  CardHeader,
  CardTitle,
  CardContent,
  Button,
  Input,
  Label,
  Badge,
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
  Separator,
  RadioGroup,
  RadioGroupItem,
} from "@/components/ui";
import {
  listMcpServersCmd,
  createMcpServerCmd,
  updateMcpServerCmd,
  deleteMcpServerCmd,
  toggleMcpServerCmd,
  discoverMcpServerCmd,
  getMcpServerStatusCmd,
  initiateMcpOauthCmd,
  type McpServer,
  type McpServerStatus,
  type CreateMcpServerRequest,
  type UpdateMcpServerRequest,
} from "@/lib/tauriCommands";

function timeAgo(iso?: string): string {
  if (!iso) return "Never";
  const diff = Date.now() - new Date(iso).getTime();
  if (diff < 60_000) return "Just now";
  const mins = Math.floor(diff / 60_000);
  if (mins < 60) return `${mins}m ago`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

function parseTransportConfig(config: string): { command: string; args: string[] } | null {
  try {
    const parsed = JSON.parse(config);
    return { command: parsed.command ?? "", args: parsed.args ?? [] };
  } catch {
    return null;
  }
}

type StatusKey = McpServerStatus["status"];

const statusColors: Record<StatusKey, string> = {
  connected: "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200",
  pending: "bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200",
  error: "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
  unreachable: "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
};

interface ServerForm {
  name: string;
  url: string;
  transport_type: "stdio" | "http";
  command: string;
  args: string;
  auth_type: "none" | "api_key" | "bearer" | "oauth2";
  auth_value: string;
  enabled: boolean;
}

const emptyForm: ServerForm = {
  name: "",
  url: "",
  transport_type: "http",
  command: "",
  args: "",
  auth_type: "none",
  auth_value: "",
  enabled: true,
};

export default function MCPServers() {
  const [servers, setServers] = useState<McpServer[]>([]);
  const [statuses, setStatuses] = useState<Record<string, McpServerStatus>>({});
  const [discovering, setDiscovering] = useState<Record<string, boolean>>({});
  const [editServer, setEditServer] = useState<McpServer | null>(null);
  const [isAdding, setIsAdding] = useState(false);
  const [form, setForm] = useState<ServerForm>({ ...emptyForm });
  const [deleteConfirm, setDeleteConfirm] = useState<string | null>(null);

  const loadServers = useCallback(async () => {
    try {
      const list = await listMcpServersCmd();
      setServers(list);
      for (const server of list) {
        getMcpServerStatusCmd(server.id)
          .then((s) => setStatuses((prev) => ({ ...prev, [server.id]: s })))
          .catch(() => {});
      }
    } catch (err) {
      console.error("Failed to load MCP servers:", err);
    }
  }, []);

  useEffect(() => {
    loadServers();
  }, [loadServers]);

  const handleDiscover = async (id: string) => {
    setDiscovering((prev) => ({ ...prev, [id]: true }));
    try {
      const status = await discoverMcpServerCmd(id);
      setStatuses((prev) => ({ ...prev, [id]: status }));
      const updated = await listMcpServersCmd();
      setServers(updated);
    } catch (err) {
      console.error("Discovery failed:", err);
    } finally {
      setDiscovering((prev) => ({ ...prev, [id]: false }));
    }
  };

  const handleToggle = async (server: McpServer) => {
    try {
      await toggleMcpServerCmd(server.id, !server.enabled);
      setServers((prev) =>
        prev.map((s) => (s.id === server.id ? { ...s, enabled: !s.enabled } : s))
      );
    } catch (err) {
      console.error("Toggle failed:", err);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await deleteMcpServerCmd(id);
      setServers((prev) => prev.filter((s) => s.id !== id));
      setDeleteConfirm(null);
    } catch (err) {
      console.error("Delete failed:", err);
    }
  };

  const startAdd = () => {
    setForm({ ...emptyForm });
    setEditServer(null);
    setIsAdding(true);
  };

  const startEdit = (server: McpServer) => {
    const parsed = parseTransportConfig(server.transport_config);
    setForm({
      name: server.name,
      url: server.url,
      transport_type: server.transport_type,
      command: parsed?.command ?? "",
      args: parsed?.args.join(" ") ?? "",
      auth_type: server.auth_type,
      auth_value: "",
      enabled: server.enabled,
    });
    setEditServer(server);
    setIsAdding(true);
  };

  const handleCancel = () => {
    setIsAdding(false);
    setEditServer(null);
    setForm({ ...emptyForm });
  };

  const handleSave = async () => {
    if (!form.name) return;
    if (form.transport_type === "http" && !form.url) return;
    if (form.transport_type === "stdio" && !form.command) return;

    const transportConfig =
      form.transport_type === "stdio"
        ? JSON.stringify({ command: form.command, args: form.args.split(/\s+/).filter(Boolean) })
        : "{}";

    const url = form.transport_type === "http" ? form.url : "";

    try {
      if (editServer) {
        const request: UpdateMcpServerRequest = {
          name: form.name,
          url,
          transport_type: form.transport_type,
          transport_config: transportConfig,
          auth_type: form.auth_type,
          enabled: form.enabled,
        };
        if (form.auth_value) {
          request.auth_value = form.auth_value;
        }
        await updateMcpServerCmd(editServer.id, request);
      } else {
        const request: CreateMcpServerRequest = {
          name: form.name,
          url,
          transport_type: form.transport_type,
          transport_config: transportConfig,
          auth_type: form.auth_type,
          auth_value: form.auth_value || undefined,
          enabled: form.enabled,
        };
        await createMcpServerCmd(request);
      }
      handleCancel();
      loadServers();
    } catch (err) {
      console.error("Failed to save MCP server:", err);
    }
  };

  const handleOAuth = async (id: string) => {
    try {
      await initiateMcpOauthCmd(id);
    } catch (err) {
      console.error("OAuth initiation failed:", err);
    }
  };

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">MCP Servers</h1>
          <p className="text-muted-foreground mt-1">
            Manage Model Context Protocol servers to extend AI tool capabilities.
          </p>
        </div>
        {!isAdding && (
          <Button onClick={startAdd}>
            <Plus className="w-4 h-4 mr-2" />
            Add Server
          </Button>
        )}
      </div>

      {servers.length === 0 && !isAdding && (
        <Card>
          <CardContent className="p-8 text-center">
            <Plug className="w-12 h-12 mx-auto text-muted-foreground mb-3" />
            <p className="text-muted-foreground">
              No MCP servers configured. Add one to extend AI tool capabilities.
            </p>
            <Button className="mt-3" onClick={startAdd}>
              <Plus className="w-4 h-4 mr-2" />
              Add your first server
            </Button>
          </CardContent>
        </Card>
      )}

      {servers.map((server) => {
        const status = statuses[server.id];
        const discoveryStatus = status?.status ?? server.discovery_status;
        const isDiscovering = discovering[server.id] ?? false;

        return (
          <Card key={server.id}>
            <CardContent className="p-4">
              <div className="flex items-center justify-between">
                <div className="space-y-1">
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium">{server.name}</span>
                    <Badge variant="secondary">
                      {server.transport_type}
                    </Badge>
                    <span
                      className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ${statusColors[discoveryStatus]}`}
                    >
                      {discoveryStatus === "connected" && <CheckCircle className="w-3 h-3 mr-1" />}
                      {discoveryStatus === "pending" && <Clock className="w-3 h-3 mr-1" />}
                      {(discoveryStatus === "error" || discoveryStatus === "unreachable") && (
                        <XCircle className="w-3 h-3 mr-1" />
                      )}
                      {discoveryStatus}
                    </span>
                    {!server.enabled && (
                      <Badge variant="outline">Disabled</Badge>
                    )}
                  </div>
                  <p className="text-xs text-muted-foreground">
                    {server.transport_type === "http" ? server.url : (() => {
                      const parsed = parseTransportConfig(server.transport_config);
                      return parsed ? `${parsed.command} ${parsed.args.join(" ")}` : server.transport_config;
                    })()}
                    {" | "}
                    Last discovered: {timeAgo(status?.last_discovered_at ?? server.last_discovered_at)}
                    {status && ` | Tools: ${status.tool_count} | Resources: ${status.resource_count}`}
                  </p>
                  {(status?.error || server.discovery_error) && (
                    <p className="text-xs text-destructive">
                      {status?.error ?? server.discovery_error}
                    </p>
                  )}
                </div>
                <div className="flex items-center gap-1">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleToggle(server)}
                    title={server.enabled ? "Disable" : "Enable"}
                  >
                    <span className={`text-xs ${server.enabled ? "text-green-600" : "text-muted-foreground"}`}>
                      {server.enabled ? "ON" : "OFF"}
                    </span>
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleDiscover(server.id)}
                    disabled={isDiscovering}
                    title="Discover Now"
                  >
                    <RefreshCw className={`w-3 h-3 ${isDiscovering ? "animate-spin" : ""}`} />
                  </Button>
                  <Button variant="ghost" size="sm" onClick={() => startEdit(server)}>
                    <Pencil className="w-3 h-3" />
                  </Button>
                  {deleteConfirm === server.id ? (
                    <div className="flex items-center gap-1">
                      <Button
                        variant="destructive"
                        size="sm"
                        onClick={() => handleDelete(server.id)}
                      >
                        Confirm
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => setDeleteConfirm(null)}
                      >
                        Cancel
                      </Button>
                    </div>
                  ) : (
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => setDeleteConfirm(server.id)}
                    >
                      <Trash2 className="w-3 h-3 text-destructive" />
                    </Button>
                  )}
                </div>
              </div>
            </CardContent>
          </Card>
        );
      })}

      {isAdding && (
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">
              {editServer ? "Edit Server" : "Add Server"}
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label>Name</Label>
              <Input
                value={form.name}
                onChange={(e) => setForm({ ...form, name: e.target.value })}
                placeholder="My MCP Server"
              />
            </div>

            <div className="space-y-2">
              <Label>Transport Type</Label>
              <RadioGroup
                value={form.transport_type}
                onValueChange={(v) =>
                  setForm({ ...form, transport_type: v as "stdio" | "http" })
                }
                className="flex gap-4"
              >
                <div className="flex items-center gap-2">
                  <RadioGroupItem value="stdio" />
                  <Label>stdio</Label>
                </div>
                <div className="flex items-center gap-2">
                  <RadioGroupItem value="http" />
                  <Label>HTTP</Label>
                </div>
              </RadioGroup>
            </div>

            {form.transport_type === "stdio" && (
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>Command</Label>
                  <Input
                    value={form.command}
                    onChange={(e) => setForm({ ...form, command: e.target.value })}
                    placeholder="/usr/local/bin/mcp-server"
                  />
                </div>
                <div className="space-y-2">
                  <Label>Arguments</Label>
                  <Input
                    value={form.args}
                    onChange={(e) => setForm({ ...form, args: e.target.value })}
                    placeholder="--port 8080 --verbose"
                  />
                  <p className="text-xs text-muted-foreground">Space-separated arguments</p>
                </div>
              </div>
            )}

            {form.transport_type === "http" && (
              <div className="space-y-2">
                <Label>URL</Label>
                <Input
                  value={form.url}
                  onChange={(e) => setForm({ ...form, url: e.target.value })}
                  placeholder="http://localhost:3001/mcp"
                />
              </div>
            )}

            <Separator />

            <div className="space-y-2">
              <Label>Authentication</Label>
              <Select
                value={form.auth_type}
                onValueChange={(v) =>
                  setForm({ ...form, auth_type: v as ServerForm["auth_type"] })
                }
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="none">None</SelectItem>
                  <SelectItem value="api_key">API Key</SelectItem>
                  <SelectItem value="bearer">Bearer Token</SelectItem>
                  <SelectItem value="oauth2">OAuth2</SelectItem>
                </SelectContent>
              </Select>
            </div>

            {(form.auth_type === "api_key" || form.auth_type === "bearer") && (
              <div className="space-y-2">
                <Label>{form.auth_type === "api_key" ? "API Key" : "Bearer Token"}</Label>
                <Input
                  type="password"
                  value={form.auth_value}
                  onChange={(e) => setForm({ ...form, auth_value: e.target.value })}
                  placeholder={editServer ? "Leave blank to keep existing" : "Enter value"}
                />
              </div>
            )}

            {form.auth_type === "oauth2" && editServer && (
              <div className="space-y-2">
                <Button variant="outline" onClick={() => handleOAuth(editServer.id)}>
                  Authenticate via Browser
                </Button>
                <p className="text-xs text-muted-foreground">
                  Opens a browser window to complete OAuth2 authentication.
                </p>
              </div>
            )}

            <Separator />

            <div className="flex items-center gap-2">
              <Button onClick={handleSave}>Save</Button>
              <Button variant="ghost" onClick={handleCancel}>
                Cancel
              </Button>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
