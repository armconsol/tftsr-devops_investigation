import React from "react";
import { Search } from "lucide-react";
import { Input } from "@/components/ui";
import { Button } from "@/components/ui";

interface SearchBarProps {
  query: string;
  onQueryChange: (query: string) => void;
  placeholder?: string;
  showClear?: boolean;
  onClear?: () => void;
}

export function SearchBar({ query, onQueryChange, placeholder = "Search...", showClear = true, onClear }: SearchBarProps) {
  const [isFocused, setIsFocused] = React.useState(false);

  const handleClear = () => {
    onQueryChange("");
    onClear?.();
  };

  return (
    <div className={`flex items-center gap-2 px-3 py-2 rounded-md border transition-colors ${isFocused ? "border-primary ring-1 ring-primary" : "border-input"}`}>
      <Search className="w-4 h-4 text-muted-foreground" />
      <Input
        type="text"
        value={query}
        onChange={(e) => onQueryChange(e.target.value)}
        onFocus={() => setIsFocused(true)}
        onBlur={() => setIsFocused(false)}
        placeholder={placeholder}
        className="border-none shadow-none focus-visible:ring-0 py-0 px-2 flex-1"
      />
      {showClear && query && (
        <Button variant="ghost" size="sm" onClick={handleClear} className="h-6 w-6 p-0">
          <Search className="w-3 h-3 rotate-45" />
        </Button>
      )}
    </div>
  );
}
