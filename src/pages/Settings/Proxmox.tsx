import React, { useEffect, useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { Switch } from '@/components/ui/index';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/index';
import { Button } from '@/components/ui/index';

export function ProxmoxSettings() {
  const [defaultPort, setDefaultPort] = useState<string>('8006');
  const [connectionTimeout, setConnectionTimeout] = useState<string>('30');
  const [retryAttempts, setRetryAttempts] = useState<string>('3');
  const [verifyCertificates, setVerifyCertificates] = useState(true);
  const [enableCaching, setEnableCaching] = useState(true);
  const [enableDebug, setEnableDebug] = useState(false);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    setDefaultPort(localStorage.getItem('proxmox_default_port') ?? '8006');
    setConnectionTimeout(localStorage.getItem('proxmox_connection_timeout') ?? '30');
    setRetryAttempts(localStorage.getItem('proxmox_retry_attempts') ?? '3');
    setVerifyCertificates((localStorage.getItem('proxmox_verify_certificates') ?? 'true') === 'true');
    setEnableCaching((localStorage.getItem('proxmox_enable_caching') ?? 'true') === 'true');
    setEnableDebug((localStorage.getItem('proxmox_enable_debug') ?? 'false') === 'true');
  }, []);

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Proxmox Settings</h1>
        <p className="text-muted-foreground">Default settings for Proxmox cluster connections</p>
      </div>

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

      <div className="flex items-center justify-end space-x-2 pt-4">
        <Button
          variant="outline"
          onClick={() => {
            ['proxmox_default_port', 'proxmox_connection_timeout', 'proxmox_retry_attempts',
             'proxmox_verify_certificates', 'proxmox_enable_caching', 'proxmox_enable_debug']
              .forEach((k) => localStorage.removeItem(k));
            setDefaultPort('8006');
            setConnectionTimeout('30');
            setRetryAttempts('3');
            setVerifyCertificates(true);
            setEnableCaching(true);
            setEnableDebug(false);
          }}
        >
          Reset to Defaults
        </Button>
        <Button
          onClick={() => {
            localStorage.setItem('proxmox_default_port', defaultPort);
            localStorage.setItem('proxmox_connection_timeout', connectionTimeout);
            localStorage.setItem('proxmox_retry_attempts', retryAttempts);
            localStorage.setItem('proxmox_verify_certificates', String(verifyCertificates));
            localStorage.setItem('proxmox_enable_caching', String(enableCaching));
            localStorage.setItem('proxmox_enable_debug', String(enableDebug));
            setSaved(true);
            setTimeout(() => setSaved(false), 2000);
          }}
        >
          Save Settings
        </Button>
        {saved && (
          <span className="text-sm text-green-600">Settings saved</span>
        )}
      </div>
    </div>
  );
}
