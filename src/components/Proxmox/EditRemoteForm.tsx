import React, { useState } from 'react';
import { Button } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/index';

interface RemoteConfig {
  id: string;
  name: string;
  url: string;
  username: string;
  type: 'pve' | 'pbs';
  status: string;
}

interface EditRemoteFormProps {
  remote: RemoteConfig;
  onSave: (config: RemoteConfig) => void;
  onCancel: () => void;
}

export function EditRemoteForm({ remote, onSave, onCancel }: EditRemoteFormProps) {
  const [config, setConfig] = useState<RemoteConfig>({
    id: remote.id,
    name: remote.name,
    url: remote.url,
    username: remote.username,
    type: remote.type,
    status: remote.status,
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
      await onSave(config);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update remote');
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
            disabled={loading}
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="url">URL</Label>
          <Input
            id="url"
            value={config.url}
            onChange={(e) => setConfig({ ...config, url: e.target.value })}
            disabled={loading}
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="username">Username</Label>
          <Input
            id="username"
            value={config.username}
            onChange={(e) => setConfig({ ...config, username: e.target.value })}
            disabled={loading}
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="type">Type</Label>
          <Input
            id="type"
            value={config.type.toUpperCase()}
            disabled
            className="bg-muted"
          />
          <p className="text-xs text-muted-foreground">
            Type cannot be changed after creation
          </p>
        </div>

        <div className="space-y-2">
          <Label htmlFor="status">Status</Label>
          <Input
            id="status"
            value={config.status}
            disabled
            className="bg-muted"
          />
        </div>

        <div className="flex justify-end space-x-2 pt-4">
          <Button type="button" variant="outline" onClick={onCancel} disabled={loading}>
            Cancel
          </Button>
          <Button type="submit" disabled={loading}>
            {loading ? 'Saving...' : 'Save Changes'}
          </Button>
        </div>
      </div>
    </form>
  );
}
