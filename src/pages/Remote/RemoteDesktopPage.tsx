import React, { useState, useEffect } from "react";
import {
  Plus,
  Edit,
  Trash2,
  Monitor,
  TestTube,
  Play,
  X,
  Save,
  Eye,
  EyeOff,
  Laptop,
} from "lucide-react";
import {
  RemoteConnection,
  RemoteConnectionSummary,
  NewRemoteConnection,
  RemoteConnectionUpdate,
  Protocol,
  addRemoteConnectionCmd,
  updateRemoteConnectionCmd,
  removeRemoteConnectionCmd,
  listRemoteConnectionsCmd,
  testRemoteConnectionCmd,
  connectRemoteCmd,
} from "@/lib/tauriCommands";
import { Button } from "@/components/ui";
import { Input } from "@/components/ui";
import { Label } from "@/components/ui";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui";
import { Badge } from "@/components/ui";
import { toast } from "sonner";

export default function RemoteDesktopPage() {
  const [connections, setConnections] = useState<RemoteConnectionSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [editingConnection, setEditingConnection] = useState<RemoteConnection | null>(null);
  const [showPassword, setShowPassword] = useState(false);
  const [testingId, setTestingId] = useState<string | null>(null);
  const [connectingId, setConnectingId] = useState<string | null>(null);

  // Form state
  const [formData, setFormData] = useState<NewRemoteConnection>({
    name: "",
    protocol: "rdp",
    hostname: "",
    port: 3389,
    username: "",
    password: "",
    domain: "",
    resolution: "1280x800",
    color_depth: 32,
    clipboard_sync: true,
    drive_redirect: false,
    multi_monitor: false,
    compression: true,
    quality: 80,
  });

  useEffect(() => {
    loadConnections();
  }, []);

  const loadConnections = async () => {
    setLoading(true);
    try {
      const data = await listRemoteConnectionsCmd();
      setConnections(data);
    } catch (error) {
      console.error("Failed to load remote connections:", error);
      toast.error("Failed to load remote connections");
    } finally {
      setLoading(false);
    }
  };

  const resetForm = () => {
    setFormData({
      name: "",
      protocol: "rdp",
      hostname: "",
      port: 3389,
      username: "",
      password: "",
      domain: "",
      resolution: "1280x800",
      color_depth: 32,
      clipboard_sync: true,
      drive_redirect: false,
      multi_monitor: false,
      compression: true,
      quality: 80,
    });
    setEditingConnection(null);
  };

  const openAddDialog = () => {
    resetForm();
    setDialogOpen(true);
  };

  const openEditDialog = (connection: RemoteConnectionSummary) => {
    setFormData({
      name: connection.name,
      protocol: connection.protocol,
      hostname: connection.hostname,
      port: connection.port,
      username: connection.username,
      password: "",
      domain: "",
      resolution: "1280x800",
      color_depth: 32,
      clipboard_sync: true,
      drive_redirect: false,
      multi_monitor: false,
      compression: true,
      quality: 80,
    });
    setEditingConnection(connection as unknown as RemoteConnection);
    setDialogOpen(true);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      if (editingConnection) {
        await updateRemoteConnectionCmd(editingConnection.id, formData as unknown as RemoteConnectionUpdate);
        toast.success("Remote connection updated successfully");
      } else {
        await addRemoteConnectionCmd(formData);
        toast.success("Remote connection created successfully");
      }
      setDialogOpen(false);
      loadConnections();
    } catch (error) {
      console.error("Failed to save remote connection:", error);
      toast.error("Failed to save remote connection");
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm("Are you sure you want to delete this remote connection?")) {
      return;
    }
    try {
      await removeRemoteConnectionCmd(id);
      toast.success("Remote connection deleted successfully");
      loadConnections();
    } catch (error) {
      console.error("Failed to delete remote connection:", error);
      toast.error("Failed to delete remote connection");
    }
  };

  const handleTest = async (id: string) => {
    setTestingId(id);
    try {
      const result = await testRemoteConnectionCmd(id);
      if (result) {
        toast.success("Connection test successful");
      } else {
        toast.error("Connection test failed");
      }
    } catch (error) {
      console.error("Failed to test connection:", error);
      toast.error("Failed to test connection");
    } finally {
      setTestingId(null);
    }
  };

  const handleConnect = async (id: string) => {
    setConnectingId(id);
    try {
      const wsUrl = await connectRemoteCmd(id);
      toast.success(`Connected to remote desktop. WebSocket URL: ${wsUrl}`);
      // In a real implementation, this would open a WebSocket connection
      // and display the remote desktop in a viewer component
      console.log("WebSocket URL for remote desktop:", wsUrl);
    } catch (error) {
      console.error("Failed to connect to remote desktop:", error);
      toast.error("Failed to connect to remote desktop");
    } finally {
      setConnectingId(null);
    }
  };

  const getProtocolIcon = (protocol: Protocol) => {
    switch (protocol) {
      case "rdp":
        return <Laptop className="w-4 h-4" />;
      case "vnc":
        return <Monitor className="w-4 h-4" />;
      default:
        return <Monitor className="w-4 h-4" />;
    }
  };

  const getPortForProtocol = (protocol: Protocol) => {
    return protocol === "rdp" ? 3389 : 5900;
  };

  return (
    <div className="p-6">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold">Remote Desktop</h1>
          <p className="text-muted-foreground mt-1">
            Manage and connect to remote desktop sessions
          </p>
        </div>
        <Button onClick={openAddDialog}>
          <Plus className="w-4 h-4 mr-2" />
          Add Connection
        </Button>
      </div>

      {loading ? (
        <div className="flex items-center justify-center py-12">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
        </div>
      ) : connections.length === 0 ? (
        <Card>
          <CardContent className="pt-6 text-center">
            <Monitor className="w-12 h-12 mx-auto text-muted-foreground mb-4" />
            <h3 className="text-lg font-semibold mb-2">No Remote Connections</h3>
            <p className="text-muted-foreground mb-4">
              Get started by adding your first remote desktop connection.
            </p>
            <Button onClick={openAddDialog}>
              <Plus className="w-4 h-4 mr-2" />
              Add Connection
            </Button>
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {connections.map((connection) => (
            <Card key={connection.id} className="relative">
              <CardHeader className="pb-3">
                <div className="flex items-center justify-between">
                  <CardTitle className="flex items-center gap-2 text-lg">
                    {getProtocolIcon(connection.protocol)}
                    {connection.name}
                  </CardTitle>
                  <Badge variant="outline">{connection.protocol.toUpperCase()}</Badge>
                </div>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="text-sm">
                  <span className="text-muted-foreground">Host:</span>{" "}
                  <span className="font-medium">{connection.hostname}:{connection.port}</span>
                </div>
                {connection.username && (
                  <div className="text-sm">
                    <span className="text-muted-foreground">User:</span>{" "}
                    <span className="font-medium">{connection.username}</span>
                  </div>
                )}
                <div className="text-sm">
                  <span className="text-muted-foreground">Status:</span>{" "}
                  <span className="font-medium capitalize">{connection.status}</span>
                </div>
                {connection.last_connected_at && (
                  <div className="text-xs text-muted-foreground">
                    Last connected: {new Date(connection.last_connected_at).toLocaleString()}
                  </div>
                )}
                <div className="flex gap-2 pt-2">
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => handleTest(connection.id)}
                    disabled={testingId === connection.id}
                  >
                    <TestTube className="w-3 h-3 mr-1" />
                    {testingId === connection.id ? "Testing..." : "Test"}
                  </Button>
                  <Button
                    size="sm"
                    onClick={() => handleConnect(connection.id)}
                    disabled={connectingId === connection.id}
                  >
                    <Play className="w-3 h-3 mr-1" />
                    {connectingId === connection.id ? "Connecting..." : "Connect"}
                  </Button>
                </div>
                <div className="flex gap-2 pt-1 border-t">
                  <Button
                    size="sm"
                    variant="ghost"
                    className="flex-1"
                    onClick={() => openEditDialog(connection)}
                  >
                    <Edit className="w-3 h-3 mr-1" />
                    Edit
                  </Button>
                  <Button
                    size="sm"
                    variant="ghost"
                    className="flex-1 text-destructive"
                    onClick={() => handleDelete(connection.id)}
                  >
                    <Trash2 className="w-3 h-3 mr-1" />
                    Delete
                  </Button>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {/* Add/Edit Dialog */}
      <Dialog open={dialogOpen} onOpenChange={(open: boolean) => {
        setDialogOpen(open);
        if (!open) resetForm();
      }}>
        <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>
              {editingConnection ? "Edit Remote Connection" : "Add Remote Connection"}
            </DialogTitle>
          </DialogHeader>
          <form onSubmit={handleSubmit}>
            <div className="grid gap-4 py-4">
              {/* Basic Settings */}
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="name">Connection Name</Label>
                  <Input
                    id="name"
                    value={formData.name}
                    onChange={(e: React.ChangeEvent<HTMLInputElement>) => setFormData({ ...formData, name: e.target.value })}
                    placeholder="My Remote Server"
                    required
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="protocol">Protocol</Label>
                  <Select
                    value={formData.protocol}
                    onValueChange={(value: string) =>
                      setFormData({ ...formData, protocol: value as Protocol, port: getPortForProtocol(value as Protocol) })
                    }
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="Select protocol" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="rdp">RDP (Remote Desktop Protocol)</SelectItem>
                      <SelectItem value="vnc">VNC (Virtual Network Computing)</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="hostname">Hostname or IP</Label>
                  <Input
                    id="hostname"
                    value={formData.hostname}
                    onChange={(e: React.ChangeEvent<HTMLInputElement>) => setFormData({ ...formData, hostname: e.target.value })}
                    placeholder="192.168.1.100"
                    required
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="port">Port</Label>
                  <Input
                    id="port"
                    type="number"
                    value={formData.port}
                    onChange={(e: React.ChangeEvent<HTMLInputElement>) => setFormData({ ...formData, port: parseInt(e.target.value) || 3389 })}
                    required
                  />
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="username">Username</Label>
                  <Input
                    id="username"
                    value={formData.username}
                    onChange={(e: React.ChangeEvent<HTMLInputElement>) => setFormData({ ...formData, username: e.target.value })}
                    placeholder="admin"
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="domain">Domain (optional)</Label>
                  <Input
                    id="domain"
                    value={formData.domain}
                    onChange={(e: React.ChangeEvent<HTMLInputElement>) => setFormData({ ...formData, domain: e.target.value })}
                    placeholder="WORKGROUP"
                  />
                </div>
              </div>

              <div className="space-y-2">
                <Label htmlFor="password">Password</Label>
                <div className="relative">
                  <Input
                    id="password"
                    type={showPassword ? "text" : "password"}
                    value={formData.password}
                    onChange={(e: React.ChangeEvent<HTMLInputElement>) => setFormData({ ...formData, password: e.target.value })}
                    placeholder={editingConnection ? "••••••••" : "Enter password"}
                    required={!editingConnection}
                  />
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    className="absolute right-0 top-0 h-full px-3"
                    onClick={() => setShowPassword(!showPassword)}
                  >
                    {showPassword ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                  </Button>
                </div>
              </div>

              {/* Display Settings */}
              <div className="border-t pt-4">
                <h4 className="font-medium mb-3">Display Settings</h4>
                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label htmlFor="resolution">Resolution</Label>
                    <Select
                      value={formData.resolution}
                      onValueChange={(value: string) => setFormData({ ...formData, resolution: value })}
                    >
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="800x600">800x600</SelectItem>
                        <SelectItem value="1024x768">1024x768</SelectItem>
                        <SelectItem value="1280x800">1280x800</SelectItem>
                        <SelectItem value="1280x1024">1280x1024</SelectItem>
                        <SelectItem value="1600x900">1600x900</SelectItem>
                        <SelectItem value="1920x1080">1920x1080</SelectItem>
                        <SelectItem value="2560x1440">2560x1440</SelectItem>
                        <SelectItem value="3840x2160">3840x2160 (4K)</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="color_depth">Color Depth</Label>
                    <Select
                      value={formData.color_depth?.toString()}
                      onValueChange={(value: string) => setFormData({ ...formData, color_depth: parseInt(value) })}
                    >
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="16">16-bit (High Color)</SelectItem>
                        <SelectItem value="24">24-bit (True Color)</SelectItem>
                        <SelectItem value="32">32-bit (True Color)</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                </div>
              </div>

              {/* Advanced Settings */}
              <div className="border-t pt-4">
                <h4 className="font-medium mb-3">Advanced Settings</h4>
                <div className="grid grid-cols-2 gap-4">
                  <div className="flex items-center space-x-2">
                    <input
                      type="checkbox"
                      id="clipboard_sync"
                      checked={formData.clipboard_sync}
                      onChange={(e) => setFormData({ ...formData, clipboard_sync: e.target.checked })}
                      className="rounded"
                    />
                    <Label htmlFor="clipboard_sync" className="font-normal">Sync Clipboard</Label>
                  </div>
                  <div className="flex items-center space-x-2">
                    <input
                      type="checkbox"
                      id="drive_redirect"
                      checked={formData.drive_redirect}
                      onChange={(e) => setFormData({ ...formData, drive_redirect: e.target.checked })}
                      className="rounded"
                    />
                    <Label htmlFor="drive_redirect" className="font-normal">Drive Redirect</Label>
                  </div>
                  <div className="flex items-center space-x-2">
                    <input
                      type="checkbox"
                      id="multi_monitor"
                      checked={formData.multi_monitor}
                      onChange={(e) => setFormData({ ...formData, multi_monitor: e.target.checked })}
                      className="rounded"
                    />
                    <Label htmlFor="multi_monitor" className="font-normal">Multi-Monitor</Label>
                  </div>
                  <div className="flex items-center space-x-2">
                    <input
                      type="checkbox"
                      id="compression"
                      checked={formData.compression}
                      onChange={(e) => setFormData({ ...formData, compression: e.target.checked })}
                      className="rounded"
                    />
                    <Label htmlFor="compression" className="font-normal">Compression</Label>
                  </div>
                </div>
                <div className="space-y-2 mt-4">
                  <Label htmlFor="quality">Quality ({formData.quality}%)</Label>
                  <input
                    type="range"
                    id="quality"
                    min="10"
                    max="100"
                    value={formData.quality}
                    onChange={(e) => setFormData({ ...formData, quality: parseInt(e.target.value) })}
                    className="w-full"
                  />
                </div>
              </div>
            </div>
            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => setDialogOpen(false)}>
                Cancel
              </Button>
              <Button type="submit">
                <Save className="w-4 h-4 mr-2" />
                {editingConnection ? "Update" : "Create"}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </div>
  );
}
