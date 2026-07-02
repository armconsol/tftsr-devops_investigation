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
}

const DB_TYPES = [
  { value: 'postgres', label: 'PostgreSQL', defaultPort: 5432 },
  { value: 'mysql', label: 'MySQL', defaultPort: 3306 },
  { value: 'mongodb', label: 'MongoDB', defaultPort: 27017 },
  { value: 'redis', label: 'Redis', defaultPort: 6379 },
  { value: 'cassandra', label: 'Cassandra', defaultPort: 9042 },
];

export function ConnectionForm({ connection, onSubmit, onCancel, isLoading }: ConnectionFormProps) {
  const [formData, setFormData] = useState<ConnectionFormData>({
    name: connection?.name || '',
    db_type: connection?.db_type || 'postgres',
    host: connection?.host || 'localhost',
    port: connection?.port || 5432,
    database_name: connection?.database_name || '',
    username: connection?.username || '',
    password: '',
    ssl_enabled: connection?.ssl_enabled || false,
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
