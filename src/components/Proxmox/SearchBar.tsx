import React from 'react';
import { Input } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Search } from 'lucide-react';

interface SearchBarProps {
  value: string;
  onChange?: (value: string) => void;
  onSearch?: (value: string) => void;
  placeholder?: string;
  isLoading?: boolean;
}

export function SearchBar({
  value,
  onChange,
  onSearch,
  placeholder = 'Search resources...',
  isLoading,
}: SearchBarProps) {
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      onSearch?.(value);
    }
  };

  return (
    <div className="flex items-center space-x-2">
      <div className="relative flex-1">
        <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
        <Input
          type="text"
          value={value}
          onChange={(e) => onChange?.(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={placeholder}
          className="pl-9"
          disabled={isLoading}
        />
      </div>
      {onSearch && (
        <Button size="sm" onClick={() => onSearch(value)} disabled={isLoading}>
          Search
        </Button>
      )}
    </div>
  );
}
