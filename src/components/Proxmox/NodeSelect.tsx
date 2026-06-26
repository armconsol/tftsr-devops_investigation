import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/index";

interface NodeSelectProps {
  nodeNames: string[];
  value: string;
  onChange: (node: string) => void;
  loading?: boolean;
  disabled?: boolean;
  className?: string;
}

/**
 * Dropdown for selecting a Proxmox node (host) within the currently selected
 * datacenter. Replaces the previous free-text node fields so users no longer
 * have to guess node names (which differ per datacenter).
 */
export function NodeSelect({
  nodeNames,
  value,
  onChange,
  loading = false,
  disabled = false,
  className = "w-44 h-8 text-sm",
}: NodeSelectProps) {
  const isEmpty = nodeNames.length === 0;
  const inactive = disabled || loading || isEmpty;

  if (inactive) {
    return (
      <button
        type="button"
        disabled
        className={`flex items-center justify-between rounded-md border border-input bg-background px-3 py-2 text-sm text-muted-foreground disabled:cursor-not-allowed disabled:opacity-50 ${className}`}
      >
        {loading ? "Loading nodes…" : isEmpty ? "No nodes" : "Select node"}
      </button>
    );
  }

  return (
    <Select value={value} onValueChange={onChange}>
      <SelectTrigger className={className}>
        <SelectValue placeholder="Select node" />
      </SelectTrigger>
      <SelectContent>
        {nodeNames.map((name) => (
          <SelectItem key={name} value={name}>
            {name}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}
