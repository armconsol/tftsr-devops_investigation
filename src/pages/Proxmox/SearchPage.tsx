import React, { useState } from 'react';
import { Card, CardContent } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Input } from '@/components/ui/index';
import { Badge } from '@/components/ui/index';
import { Search, Server, HardDrive, Cpu, Database } from 'lucide-react';
import { searchResources, listProxmoxClusters } from '@/lib/proxmoxClient';
import type { SearchResult } from '@/lib/proxmoxClient';

const TYPE_ICONS: Record<string, React.ElementType> = {
  vm: Cpu,
  container: HardDrive,
  node: Server,
  storage: Database,
  pool: Server,
};

export function ProxmoxSearchPage() {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [searching, setSearching] = useState(false);
  const [searched, setSearched] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSearch = async () => {
    if (!query.trim()) return;
    setSearching(true);
    setError(null);
    setSearched(false);
    try {
      const clusters = await listProxmoxClusters();
      const allResults: SearchResult[] = [];
      await Promise.all(
        clusters.map(async (c) => {
          try {
            const r = await searchResources(c.id, query);
            allResults.push(...r);
          } catch {
            // skip clusters that fail individually
          }
        })
      );
      setResults(allResults);
      setSearched(true);
    } catch (e) {
      setError(String(e));
    } finally {
      setSearching(false);
    }
  };

  // Group results by type
  const grouped = results.reduce<Record<string, SearchResult[]>>((acc, r) => {
    const bucket = acc[r.type] ?? [];
    bucket.push(r);
    acc[r.type] = bucket;
    return acc;
  }, {});

  return (
    <div className="space-y-4">
      <div>
        <h1 className="text-2xl font-bold">Search</h1>
        <p className="text-muted-foreground">Search across all Proxmox resources</p>
      </div>

      <div className="flex space-x-2">
        <Input
          placeholder="Search VMs, containers, nodes, storage..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') void handleSearch();
          }}
          className="max-w-lg"
        />
        <Button onClick={() => void handleSearch()} disabled={searching}>
          <Search className="mr-2 h-4 w-4" />
          {searching ? 'Searching...' : 'Search'}
        </Button>
      </div>

      {error && <div className="text-destructive text-sm">{error}</div>}

      {Object.entries(grouped).map(([type, items]) => {
        const Icon = TYPE_ICONS[type] ?? Server;
        return (
          <Card key={type}>
            <CardContent className="pt-4">
              <div className="flex items-center gap-2 text-sm font-semibold capitalize mb-2">
                <Icon className="h-4 w-4" />
                {type}s ({items.length})
              </div>
              <div className="space-y-1">
                {items.map((r) => (
                  <div
                    key={`${r.type}-${r.id}`}
                    className="flex items-center gap-2 p-2 rounded hover:bg-accent"
                  >
                    <Badge variant="outline" className="text-xs">
                      {r.type}
                    </Badge>
                    <span className="text-sm font-medium">{r.name}</span>
                    {r.node && (
                      <span className="text-xs text-muted-foreground">on {r.node}</span>
                    )}
                    {r.description && (
                      <span className="text-xs text-muted-foreground truncate max-w-xs">
                        — {r.description}
                      </span>
                    )}
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        );
      })}

      {searched && results.length === 0 && (
        <div className="text-muted-foreground text-sm">
          No results found for &ldquo;{query}&rdquo;
        </div>
      )}
    </div>
  );
}
