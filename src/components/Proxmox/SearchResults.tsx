import React from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';

interface SearchResult {
  id: string;
  name: string;
  type: string;
  remote: string;
  status: string;
}

interface SearchResultsProps {
  results: SearchResult[];
  onNavigate?: (result: SearchResult) => void;
}

export function SearchResults({ results, onNavigate }: SearchResultsProps) {
  return (
    <Card className="mt-4">
      <CardHeader className="pb-2">
        <CardTitle>Search Results ({results.length})</CardTitle>
      </CardHeader>
      <CardContent>
        {results.length === 0 ? (
          <div className="text-center text-muted-foreground py-8">
            <p>No results found</p>
          </div>
        ) : (
          <div className="space-y-2">
            {results.map((result) => (
              <div
                key={result.id}
                className="flex items-center justify-between p-3 rounded-md hover:bg-accent cursor-pointer"
                onClick={() => onNavigate?.(result)}
              >
                <div>
                  <div className="font-medium">{result.name}</div>
                  <div className="text-xs text-muted-foreground">
                    {result.type} • {result.remote}
                  </div>
                </div>
                <span className="text-xs text-muted-foreground">{result.status}</span>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
