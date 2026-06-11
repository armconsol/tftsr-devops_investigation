import React from 'react';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { Switch } from '@/components/ui/index';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/index';
import { Button } from '@/components/ui/index';

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

  const handleCheckUpdates = () => {
    setLastCheck(new Date().toLocaleString());
    console.log('Checking for updates...');
  };

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
              onValueChange={(value: string) => setUpdateChannel(value as 'stable' | 'pre-release')}
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
              Check Now
            </Button>
          </div>
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
