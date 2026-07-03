// Database Connection Form Component

import { useState } from 'react';
import { Button, Input, Label, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui';
import type { DatabaseConnection } from '@/stores/databaseStore';

interface ConnectionFormProps {
  connection?: DatabaseConnection;
  onSubmit: (data: ConnectionFormData) => void;
  onCancel: () => void;
  isLoading?: boolean;
}

export interface ConnectionFormData {
  name: string;
  db_type: string;
  host: string;
  port: number;
  database_name?: string;
  username: string;
  password: string;
  ssl_enabled: boolean;
  // SSH tunnel configuration
  ssh_enabled: boolean;
  ssh_hostname?: string;
  ssh_port?: number;
  ssh_username?: string;
  ssh_auth_method?: 'password' | 'key';
  ssh_password?: string;
  ssh_private_key?: string;
  ssh_key_passphrase?: string;
}

const DB_TYPES = [
  { value: 'postgresql', label: 'PostgreSQL', defaultPort: 5432 },
  { value: 'mysql', label: 'MySQL', defaultPort: 3306 },
  { value: 'mongodb', label: 'MongoDB', defaultPort: 27017 },
  { value: 'redis', label: 'Redis', defaultPort: 6379 },
  { value: 'cassandra', label: 'Cassandra', defaultPort: 9042 },
];

export function ConnectionForm({ connection, onSubmit, onCancel, isLoading }: ConnectionFormProps) {
  const [formData, setFormData] = useState<ConnectionFormData>({
    name: connection?.name || '',
    db_type: connection?.db_type || 'postgresql',
    host: connection?.host || 'localhost',
    port: connection?.port || 5432,
    database_name: connection?.database_name || '',
    username: connection?.username || '',
    password: '',
    ssl_enabled: connection?.ssl_enabled || false,
    ssh_enabled: connection?.ssh_enabled || false,
    ssh_hostname: connection?.ssh_hostname || '',
    ssh_port: connection?.ssh_port || 22,
    ssh_username: connection?.ssh_username || '',
    ssh_auth_method: connection?.ssh_auth_method as 'password' | 'key' || 'password',
    ssh_password: '',
    ssh_private_key: '',
    ssh_key_passphrase: '',
  });

  const [errors, setErrors] = useState<Record<string, string>>({});

  const validate = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!formData.name.trim()) {
      newErrors.name = 'Connection name is required';
    }
    if (!formData.host.trim()) {
      newErrors.host = 'Host is required';
    }
    if (formData.port < 1 || formData.port > 65535) {
      newErrors.port = 'Port must be between 1 and 65535';
    }
    if (!formData.username.trim()) {
      newErrors.username = 'Username is required';
    }
    if (!connection && !formData.password) {
      newErrors.password = 'Password is required for new connections';
    }

    // SSH tunnel validation
    if (formData.ssh_enabled) {
      if (!formData.ssh_hostname?.trim()) {
        newErrors.ssh_hostname = 'SSH hostname is required';
      }
      if (!formData.ssh_port || formData.ssh_port < 1 || formData.ssh_port > 65535) {
        newErrors.ssh_port = 'SSH port must be between 1 and 65535';
      }
      if (!formData.ssh_username?.trim()) {
        newErrors.ssh_username = 'SSH username is required';
      }
      if (formData.ssh_auth_method === 'key' && !formData.ssh_private_key?.trim()) {
        newErrors.ssh_private_key = 'SSH private key is required for key authentication';
      }
      if (formData.ssh_auth_method === 'password' && !formData.ssh_password?.trim()) {
        newErrors.ssh_password = 'SSH password is required for password authentication';
      }
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (validate()) {
      onSubmit(formData);
    }
  };

  const handleDbTypeChange = (value: string) => {
    const dbType = DB_TYPES.find((t) => t.value === value);
    setFormData({
      ...formData,
      db_type: value,
      port: dbType?.defaultPort || formData.port,
    });
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div>
        <Label htmlFor="name">Connection Name *</Label>
        <Input
          id="name"
          value={formData.name}
          onChange={(e) => setFormData({ ...formData, name: e.target.value })}
          placeholder="Production DB"
          className={errors.name ? 'border-red-500' : ''}
        />
        {errors.name && <p className="text-sm text-red-500 mt-1">{errors.name}</p>}
      </div>

      <div>
        <Label htmlFor="db_type">Database Type *</Label>
        <Select value={formData.db_type} onValueChange={handleDbTypeChange}>
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {DB_TYPES.map((type) => (
              <SelectItem key={type.value} value={type.value}>
                {type.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      <div className="grid grid-cols-2 gap-4">
        <div>
          <Label htmlFor="host">Host *</Label>
          <Input
            id="host"
            value={formData.host}
            onChange={(e) => setFormData({ ...formData, host: e.target.value })}
            placeholder="localhost"
            className={errors.host ? 'border-red-500' : ''}
          />
          {errors.host && <p className="text-sm text-red-500 mt-1">{errors.host}</p>}
        </div>

        <div>
          <Label htmlFor="port">Port *</Label>
          <Input
            id="port"
            type="number"
            value={formData.port}
            onChange={(e) => setFormData({ ...formData, port: parseInt(e.target.value) || 0 })}
            className={errors.port ? 'border-red-500' : ''}
          />
          {errors.port && <p className="text-sm text-red-500 mt-1">{errors.port}</p>}
        </div>
      </div>

      <div>
        <Label htmlFor="database_name">Database Name</Label>
        <Input
          id="database_name"
          value={formData.database_name}
          onChange={(e) => setFormData({ ...formData, database_name: e.target.value })}
          placeholder="Optional"
        />
      </div>

      <div>
        <Label htmlFor="username">Username *</Label>
        <Input
          id="username"
          value={formData.username}
          onChange={(e) => setFormData({ ...formData, username: e.target.value })}
          placeholder="postgres"
          className={errors.username ? 'border-red-500' : ''}
        />
        {errors.username && <p className="text-sm text-red-500 mt-1">{errors.username}</p>}
      </div>

      <div>
        <Label htmlFor="password">
          Password {!connection && '*'}
        </Label>
        <Input
          id="password"
          type="password"
          value={formData.password}
          onChange={(e) => setFormData({ ...formData, password: e.target.value })}
          placeholder={connection ? 'Leave blank to keep existing' : 'Enter password'}
          className={errors.password ? 'border-red-500' : ''}
        />
        {errors.password && <p className="text-sm text-red-500 mt-1">{errors.password}</p>}
      </div>

      <div className="flex items-center space-x-2">
        <input
          type="checkbox"
          id="ssl_enabled"
          checked={formData.ssl_enabled}
          onChange={(e) =>
            setFormData({ ...formData, ssl_enabled: e.target.checked })
          }
          className="rounded border-gray-300"
        />
        <Label htmlFor="ssl_enabled">Enable SSL/TLS</Label>
      </div>

      {/* SSH Tunnel Configuration */}
      <div className="border-t pt-4 mt-4">
        <div className="flex items-center space-x-2 mb-4">
          <input
            type="checkbox"
            id="ssh_enabled"
            checked={formData.ssh_enabled}
            onChange={(e) =>
              setFormData({ ...formData, ssh_enabled: e.target.checked })
            }
            className="rounded border-gray-300"
          />
          <Label htmlFor="ssh_enabled" className="font-semibold">Enable SSH Tunnel</Label>
        </div>

        {formData.ssh_enabled && (
          <div className="space-y-4 pl-6 border-l-2 border-gray-300">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label htmlFor="ssh_hostname">SSH Hostname *</Label>
                <Input
                  id="ssh_hostname"
                  value={formData.ssh_hostname || ''}
                  onChange={(e) => setFormData({ ...formData, ssh_hostname: e.target.value })}
                  placeholder="jump.example.com"
                  className={errors.ssh_hostname ? 'border-red-500' : ''}
                />
                {errors.ssh_hostname && <p className="text-sm text-red-500 mt-1">{errors.ssh_hostname}</p>}
              </div>

              <div>
                <Label htmlFor="ssh_port">SSH Port *</Label>
                <Input
                  id="ssh_port"
                  type="number"
                  value={formData.ssh_port || 22}
                  onChange={(e) => setFormData({ ...formData, ssh_port: parseInt(e.target.value) || 22 })}
                  className={errors.ssh_port ? 'border-red-500' : ''}
                />
                {errors.ssh_port && <p className="text-sm text-red-500 mt-1">{errors.ssh_port}</p>}
              </div>
            </div>

            <div>
              <Label htmlFor="ssh_username">SSH Username *</Label>
              <Input
                id="ssh_username"
                value={formData.ssh_username || ''}
                onChange={(e) => setFormData({ ...formData, ssh_username: e.target.value })}
                placeholder="ubuntu"
                className={errors.ssh_username ? 'border-red-500' : ''}
              />
              {errors.ssh_username && <p className="text-sm text-red-500 mt-1">{errors.ssh_username}</p>}
            </div>

            <div>
              <Label htmlFor="ssh_auth_method">Authentication Method *</Label>
              <Select 
                value={formData.ssh_auth_method || 'password'}
                onValueChange={(value) => setFormData({ ...formData, ssh_auth_method: value as 'password' | 'key' })}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="password">Password</SelectItem>
                  <SelectItem value="key">SSH Key</SelectItem>
                </SelectContent>
              </Select>
            </div>

            {formData.ssh_auth_method === 'password' && (
              <div>
                <Label htmlFor="ssh_password">SSH Password *</Label>
                <Input
                  id="ssh_password"
                  type="password"
                  value={formData.ssh_password || ''}
                  onChange={(e) => setFormData({ ...formData, ssh_password: e.target.value })}
                  placeholder="Password"
                  className={errors.ssh_password ? 'border-red-500' : ''}
                />
                {errors.ssh_password && <p className="text-sm text-red-500 mt-1">{errors.ssh_password}</p>}
              </div>
            )}

            {formData.ssh_auth_method === 'key' && (
              <>
                <div>
                  <Label htmlFor="ssh_private_key">SSH Private Key (PEM) *</Label>
                  <textarea
                    id="ssh_private_key"
                    value={formData.ssh_private_key || ''}
                    onChange={(e) => setFormData({ ...formData, ssh_private_key: e.target.value })}
                    placeholder="-----BEGIN PRIVATE KEY-----&#10;...&#10;-----END PRIVATE KEY-----"
                    className={`w-full px-3 py-2 border rounded-md font-mono text-sm ${errors.ssh_private_key ? 'border-red-500' : 'border-gray-300'}`}
                    rows={6}
                  />
                  {errors.ssh_private_key && <p className="text-sm text-red-500 mt-1">{errors.ssh_private_key}</p>}
                </div>

                <div>
                  <Label htmlFor="ssh_key_passphrase">SSH Key Passphrase (if encrypted)</Label>
                  <Input
                    id="ssh_key_passphrase"
                    type="password"
                    value={formData.ssh_key_passphrase || ''}
                    onChange={(e) => setFormData({ ...formData, ssh_key_passphrase: e.target.value })}
                    placeholder="Leave blank if key is not encrypted"
                  />
                </div>
              </>
            )}
          </div>
        )}
      </div>

      <div className="flex justify-end gap-2 pt-4">
        <Button type="button" variant="outline" onClick={onCancel} disabled={isLoading}>
          Cancel
        </Button>
        <Button type="submit" disabled={isLoading}>
          {isLoading ? 'Saving...' : connection ? 'Update' : 'Create'}
        </Button>
      </div>
    </form>
  );
}
