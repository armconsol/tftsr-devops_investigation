import React, { useState } from 'react';
import { Button } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { DialogFooter } from '@/components/ui/index';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/index';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/index';

interface RemoteConfig {
  id?: string;
  name: string;
  url: string;
  username: string;
  password?: string;
  tokenName?: string;
  tokenValue?: string;
  type: 'pve' | 'pbs';
  fingerprint?: string;
  verifyCertificate: boolean;
  description?: string;
}

interface AddRemoteFormProps {
  onAdd: (config: RemoteConfig) => void;
  onCancel: () => void;
}

export function AddRemoteForm({ onAdd, onCancel }: AddRemoteFormProps) {
  const [config, setConfig] = useState<RemoteConfig>({
    name: '',
    url: '',
    username: '',
    password: '',
    tokenName: '',
    tokenValue: '',
    type: 'pve',
    verifyCertificate: true,
    description: '',
  });
  const [error, setError] = useState<string>('');
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    if (!config.name.trim()) {
      setError('Remote name is required');
      return;
    }
    if (!config.url.trim()) {
      setError('URL is required');
      return;
    }
    if (!config.username.trim()) {
      setError('Username is required');
      return;
    }

    setLoading(true);
    try {
      await onAdd(config);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to add remote');
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit}>
      <div className="space-y-4">
        {error && (
          <Alert variant="destructive">
            <AlertTitle>Error</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        <div className="space-y-2">
          <Label htmlFor="name">Remote Name</Label>
          <Input
            id="name"
            value={config.name}
            onChange={(e) => setConfig({ ...config, name: e.target.value })}
            placeholder="e.g., Production Cluster"
            disabled={loading}
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="url">URL</Label>
          <Input
            id="url"
            value={config.url}
            onChange={(e) => setConfig({ ...config, url: e.target.value })}
            placeholder="https://pve.example.com:8006"
            disabled={loading}
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="username">Username</Label>
          <Input
            id="username"
            value={config.username}
            onChange={(e) => setConfig({ ...config, username: e.target.value })}
            placeholder="root@pam"
            disabled={loading}
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="type">Type</Label>
          <Select
            value={config.type}
            onValueChange={(value: string) =>
              setConfig({ ...config, type: value as 'pve' | 'pbs' })
            }
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="pve">Proxmox VE</SelectItem>
              <SelectItem value="pbs">Proxmox Backup Server</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-2">
          <Label htmlFor="password">Password</Label>
          <Input
            id="password"
            type="password"
            value={config.password || ''}
            onChange={(e) => setConfig({ ...config, password: e.target.value })}
            placeholder="Enter password"
            disabled={loading}
          />
          <p className="text-xs text-muted-foreground">
            Leave blank to use API token authentication
          </p>
        </div>

        <div className="space-y-2">
          <Label htmlFor="tokenName">Token Name</Label>
          <Input
            id="tokenName"
            value={config.tokenName || ''}
            onChange={(e) => setConfig({ ...config, tokenName: e.target.value })}
            placeholder="e.g., mytoken"
            disabled={loading}
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="tokenValue">Token Value</Label>
          <Input
            id="tokenValue"
            type="password"
            value={config.tokenValue || ''}
            onChange={(e) => setConfig({ ...config, tokenValue: e.target.value })}
            placeholder="Enter token value"
            disabled={loading}
          />
        </div>

        <div className="flex items-center space-x-2">
          <input
            id="verifyCertificate"
            type="checkbox"
            checked={config.verifyCertificate}
            onChange={(e) =>
              setConfig({ ...config, verifyCertificate: e.target.checked })
            }
            disabled={loading}
            className="rounded border-gray-300 text-primary focus:ring-primary"
          />
          <Label htmlFor="verifyCertificate">Verify SSL Certificate</Label>
        </div>

        <div className="space-y-2">
          <Label htmlFor="description">Description</Label>
          <Input
            id="description"
            value={config.description || ''}
            onChange={(e) => setConfig({ ...config, description: e.target.value })}
            placeholder="Optional description"
            disabled={loading}
          />
        </div>

        <DialogFooter className="flex justify-end space-x-2 pt-4">
          <Button type="button" variant="outline" onClick={onCancel} disabled={loading}>
            Cancel
          </Button>
          <Button type="submit" disabled={loading}>
            {loading ? 'Adding...' : 'Add Remote'}
          </Button>
        </DialogFooter>
      </div>
    </form>
  );
}
