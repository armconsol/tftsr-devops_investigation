import React from 'react';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { Switch } from '@/components/ui/index';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { RefreshCw, Check, AlertCircle, Loader } from 'lucide-react';
import {
  checkAppUpdatesCmd,
  installAppUpdatesCmd,
  getUpdateChannelCmd,
  setUpdateChannelCmd,
} from '@/lib/tauriCommands';

export function ProxmoxSettings() {
  const [autoUpdate, setAutoUpdate] = React.useState(true);
  const [updateChannel, setUpdateChannel] = React.useState<'stable' | 'pre-release'>('stable');
  const [autoCheck, setAutoCheck] = React.useState(true);
  const [lastCheck, setLastCheck] = React.useState<string>('Never');
  const [defaultPort, setDefaultPort] = React.useState<string>('8006');
  const [connectionTimeout, setConnectionTimeout] = React.useState<string>('30');
  const [retryAttempts, setRetryAttempts] = React.useState<string>('3');
  const [verifyCertificates, setVerifyCertificates] = React.useState(true);
  const [enableCaching, setEnableCaching] = React.useState(true);
  const [enableDebug, setEnableDebug] = React.useState(false);
  const [checking, setChecking] = React.useState(false);
  const [updateAvailable, setUpdateAvailable] = React.useState(false);
  const [error, setError] = React.useState<string | null>(null);

  const loadChannel = async () => {
    try {
      const ch = await getUpdateChannelCmd();
      setUpdateChannel(ch as 'stable' | 'pre-release');
    } catch {
      console.error('Failed to load channel');
    }
  };

  const handleCheckUpdates = async () => {
    setChecking(true);
    setError(null);
    try {
      const available = await checkAppUpdatesCmd();
      setUpdateAvailable(available);
      setLastCheck(new Date().toLocaleString());
    } catch {
      setError('Failed to check for updates');
    } finally {
      setChecking(false);
    }
  };

  const handleInstallUpdate = async () => {
    try {
      await installAppUpdatesCmd();
      setUpdateAvailable(false);
    } catch {
      setError('Failed to install update');
    }
  };

  const handleChannelChange = async (value: string) => {
    setUpdateChannel(value as 'stable' | 'pre-release');
    try {
      await setUpdateChannelCmd(value);
    } catch {
      setError('Failed to update channel');
    }
  };

  React.useEffect(() => {
    void loadChannel();
    void handleCheckUpdates();
  }, []);

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Proxmox Settings</h1>
        <p className="text-muted-foreground">Configure Proxmox Datacenter Manager integration</p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Update Management</CardTitle>
          <CardDescription>Configure how Proxmox updates are managed</CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="autoUpdate">Auto-check for updates</Label>
              <p className="text-sm text-muted-foreground">
                Automatically check for new Proxmox updates
              </p>
            </div>
            <Switch
              checked={autoUpdate}
              onCheckedChange={setAutoUpdate}
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="updateChannel">Update Channel</Label>
            <Select
              value={updateChannel}
              onValueChange={handleChannelChange}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="stable">Stable</SelectItem>
                <SelectItem value="pre-release">Pre-Release</SelectItem>
              </SelectContent>
            </Select>
            <p className="text-xs text-muted-foreground">
              {updateChannel === 'stable' 
                ? 'Receive only stable, production-ready updates' 
                : 'Receive pre-release updates with new features (may be less stable)'}
            </p>
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="autoCheck">Auto-download updates</Label>
              <p className="text-sm text-muted-foreground">
                Automatically download updates when available
              </p>
            </div>
            <Switch
              checked={autoCheck}
              onCheckedChange={setAutoCheck}
            />
          </div>

          <div className="flex items-center justify-between rounded-lg border p-4">
            <div className="space-y-0.5">
              <Label className="text-base">Last check</Label>
              <p className="text-sm text-muted-foreground">
                {lastCheck}
              </p>
            </div>
            <Button onClick={handleCheckUpdates} variant="outline">
              {checking ? (
                <>
                  <Loader className="mr-2 h-4 w-4 animate-spin" />
                  Checking...
                </>
              ) : (
                <>
                  <RefreshCw className="mr-2 h-4 w-4" />
                  Check Now
                </>
              )}
            </Button>
          </div>

          {error && (
            <div className="flex items-center space-x-2 rounded-lg bg-destructive/15 p-3 text-destructive">
              <AlertCircle className="h-4 w-4" />
              <span className="text-sm">{error}</span>
            </div>
          )}

          {updateAvailable ? (
            <div className="flex items-center justify-between rounded-lg bg-green-50 p-4 dark:bg-green-900/20">
              <div className="flex items-center space-x-3">
                <div className="rounded-full bg-green-600 p-1 text-white">
                  <Check className="h-4 w-4" />
                </div>
                <div>
                  <div className="font-semibold text-green-900 dark:text-green-100">
                    Update Available
                  </div>
                  <div className="text-sm text-green-700 dark:text-green-300">
                    A new version is ready to install
                  </div>
                </div>
              </div>
              <Button onClick={handleInstallUpdate}>
                Install Update
              </Button>
            </div>
          ) : (
            <div className="flex items-center justify-between rounded-lg bg-muted p-4">
              <div className="flex items-center space-x-3">
                <div className="rounded-full bg-muted-foreground p-1 text-background">
                  <Check className="h-4 w-4" />
                </div>
                <div>
                  <div className="font-semibold">Up to Date</div>
                  <div className="text-sm text-muted-foreground">
                    You are running the latest version
                  </div>
                </div>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Cluster Configuration</CardTitle>
          <CardDescription>Default settings for new Proxmox clusters</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="defaultPort">Default Port</Label>
            <div className="flex space-x-2">
            <Select value={defaultPort} onValueChange={setDefaultPort}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="8006">8006 (Proxmox VE)</SelectItem>
                <SelectItem value="8007">8007 (Proxmox Backup Server)</SelectItem>
              </SelectContent>
            </Select>
              <p className="text-xs text-muted-foreground self-center">
                Used when connecting to new clusters
              </p>
            </div>
          </div>

          <div className="space-y-2">
            <Label htmlFor="connectionTimeout">Connection Timeout (seconds)</Label>
            <Select value={connectionTimeout} onValueChange={setConnectionTimeout}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="10">10 seconds</SelectItem>
                <SelectItem value="30">30 seconds</SelectItem>
                <SelectItem value="60">60 seconds</SelectItem>
                <SelectItem value="120">120 seconds</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-2">
            <Label htmlFor="retryAttempts">Retry Attempts</Label>
            <Select value={retryAttempts} onValueChange={setRetryAttempts}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="1">1 attempt</SelectItem>
                <SelectItem value="3">3 attempts</SelectItem>
                <SelectItem value="5">5 attempts</SelectItem>
                <SelectItem value="10">10 attempts</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Advanced Options</CardTitle>
          <CardDescription>Advanced Proxmox integration settings</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="verifyCertificates">Verify SSL certificates</Label>
              <p className="text-sm text-muted-foreground">
                Require valid SSL certificates for cluster connections
              </p>
            </div>
            <Switch checked={verifyCertificates} onCheckedChange={setVerifyCertificates} />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="enableCaching">Enable connection caching</Label>
              <p className="text-sm text-muted-foreground">
                Reuse connections to improve performance
              </p>
            </div>
            <Switch checked={enableCaching} onCheckedChange={setEnableCaching} />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="enableDebug">Enable debug logging</Label>
              <p className="text-sm text-muted-foreground">
                Log detailed Proxmox API interactions
              </p>
            </div>
            <Switch checked={enableDebug} onCheckedChange={setEnableDebug} />
          </div>
        </CardContent>
      </Card>

      <div className="flex justify-end space-x-2 pt-4">
        <Button variant="outline">Reset to Defaults</Button>
        <Button>Save Settings</Button>
      </div>
    </div>
  );
}
