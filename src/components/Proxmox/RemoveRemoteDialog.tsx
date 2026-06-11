import React, { useState } from 'react';
import { Button } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { Label } from '@/components/ui/index';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/index';

interface RemoteConfig {
  id: string;
  name: string;
  url: string;
  type: 'pve' | 'pbs';
  status: string;
}

interface RemoveRemoteDialogProps {
  remote: RemoteConfig;
  onConfirm: () => void;
  onCancel: () => void;
}

export function RemoveRemoteDialog({ remote, onConfirm, onCancel }: RemoveRemoteDialogProps) {
  const [loading, setLoading] = useState(false);
  const [confirmText, setConfirmText] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);

    if (confirmText !== remote.name) {
      setLoading(false);
      return;
    }

    try {
      await onConfirm();
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <Alert variant="destructive">
        <AlertTitle>Warning</AlertTitle>
        <AlertDescription>
          Are you sure you want to remove remote "{remote.name}"? This action cannot be undone.
        </AlertDescription>
      </Alert>

      <div className="space-y-2">
        <Label htmlFor="confirm">
          Type <code className="font-mono">{remote.name}</code> to confirm
        </Label>
        <Input
          id="confirm"
          value={confirmText}
          onChange={(e) => setConfirmText(e.target.value)}
          placeholder={remote.name}
          disabled={loading}
        />
      </div>

      <div className="flex justify-end space-x-2 pt-4">
        <Button type="button" variant="outline" onClick={onCancel} disabled={loading}>
          Cancel
        </Button>
        <Button type="submit" variant="destructive" disabled={loading || confirmText !== remote.name}>
          {loading ? 'Removing...' : 'Remove Remote'}
        </Button>
      </div>
    </form>
  );
}
