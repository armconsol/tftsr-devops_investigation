import React from 'react';
import { Card } from '@/components/ui/index';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/index';
import { Input } from '@/components/ui/index';

interface ResourceFilterProps {
  remotes: { id: string; name: string }[];
  selectedRemote?: string;
  onRemoteChange?: (remoteId: string) => void;
  resourceTypes: { value: string; label: string }[];
  selectedType?: string;
  onTypeChange?: (type: string) => void;
  pools?: { id: string; name: string }[];
  selectedPool?: string;
  onPoolChange?: (poolId: string) => void;
  tags?: string[];
  selectedTag?: string;
  onTagChange?: (tag: string) => void;
  search?: string;
  onSearchChange?: (search: string) => void;
  onApply?: () => void;
  onClear?: () => void;
}

export function ResourceFilter({
  remotes = [],
  selectedRemote,
  onRemoteChange,
  resourceTypes = [
    { value: 'all', label: 'All Types' },
    { value: 'node', label: 'Nodes' },
    { value: 'qemu', label: 'VMs' },
    { value: 'lxc', label: 'Containers' },
    { value: 'storage', label: 'Storage' },
    { value: 'datastore', label: 'Datastores' },
    { value: 'sdn-zone', label: 'SDN Zones' },
  ],
  selectedType,
  onTypeChange,
  pools = [],
  selectedPool,
  onPoolChange,
  tags = [],
  selectedTag,
  onTagChange,
  search,
  onSearchChange,
  onApply,
  onClear,
}: ResourceFilterProps) {
  return (
    <Card className="p-4 space-y-4">
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {remotes.length > 0 && (
          <div className="space-y-2">
            <label className="text-sm font-medium">Remote</label>
            <Select
              value={selectedRemote || ''}
              onValueChange={(value) => onRemoteChange?.(value)}
            >
              <SelectTrigger>
                <SelectValue placeholder="Select remote" />
              </SelectTrigger>
              <SelectContent>
                {remotes.map((remote) => (
                  <SelectItem key={remote.id} value={remote.id}>
                    {remote.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        )}

        <div className="space-y-2">
          <label className="text-sm font-medium">Resource Type</label>
          <Select
            value={selectedType || 'all'}
            onValueChange={(value) => onTypeChange?.(value)}
          >
            <SelectTrigger>
              <SelectValue placeholder="Select type" />
            </SelectTrigger>
            <SelectContent>
              {resourceTypes.map((type) => (
                <SelectItem key={type.value} value={type.value}>
                  {type.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        {pools.length > 0 && (
          <div className="space-y-2">
            <label className="text-sm font-medium">Pool</label>
            <Select
              value={selectedPool || ''}
              onValueChange={(value) => onPoolChange?.(value)}
            >
              <SelectTrigger>
                <SelectValue placeholder="Select pool" />
              </SelectTrigger>
              <SelectContent>
                {pools.map((pool) => (
                  <SelectItem key={pool.id} value={pool.id}>
                    {pool.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        )}

        {tags.length > 0 && (
          <div className="space-y-2">
            <label className="text-sm font-medium">Tag</label>
            <Select
              value={selectedTag || ''}
              onValueChange={(value) => onTagChange?.(value)}
            >
              <SelectTrigger>
                <SelectValue placeholder="Select tag" />
              </SelectTrigger>
              <SelectContent>
                {tags.map((tag) => (
                  <SelectItem key={tag} value={tag}>
                    {tag}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        )}

        <div className="space-y-2">
          <label className="text-sm font-medium">Search</label>
          <Input
            placeholder="Search resources..."
            value={search || ''}
            onChange={(e) => onSearchChange?.(e.target.value)}
          />
        </div>
      </div>

      <div className="flex space-x-2">
        <button
          onClick={onApply}
          className="px-4 py-2 bg-primary text-primary-foreground hover:bg-primary/90 rounded-md text-sm font-medium"
        >
          Apply Filters
        </button>
        <button
          onClick={onClear}
          className="px-4 py-2 bg-secondary text-secondary-foreground hover:bg-secondary/80 rounded-md text-sm font-medium"
        >
          Clear
        </button>
      </div>
    </Card>
  );
}
