// Column Mapping Component for CSV/JSON Import

import { useState } from 'react';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui';
import { ArrowRight } from 'lucide-react';

export interface ColumnMapping {
  sourceColumn: string;
  targetColumn: string;
}

interface ColumnMapperProps {
  sourceColumns: string[];
  targetColumns: Array<{ name: string; data_type: string }>;
  initialMappings?: ColumnMapping[];
  onMappingsChange: (mappings: ColumnMapping[]) => void;
}

export function ColumnMapper({
  sourceColumns,
  targetColumns,
  initialMappings = [],
  onMappingsChange,
}: ColumnMapperProps) {
  const [mappings, setMappings] = useState<ColumnMapping[]>(() => {
    if (initialMappings.length > 0) {
      return initialMappings;
    }

    // Auto-map columns with matching names
    return sourceColumns.map((sourceCol) => {
      const matchingTarget = targetColumns.find(
        (targetCol) => targetCol.name.toLowerCase() === sourceCol.toLowerCase()
      );
      return {
        sourceColumn: sourceCol,
        targetColumn: matchingTarget?.name || '',
      };
    });
  });

  const handleMappingChange = (sourceColumn: string, targetColumn: string) => {
    const newMappings = mappings.map((mapping) =>
      mapping.sourceColumn === sourceColumn
        ? { ...mapping, targetColumn }
        : mapping
    );
    setMappings(newMappings);
    onMappingsChange(newMappings);
  };

  return (
    <div className="space-y-2">
      <div className="grid grid-cols-[1fr,auto,1fr] gap-4 items-center font-semibold text-sm pb-2 border-b">
        <div>Source Column</div>
        <div />
        <div>Target Column (Table)</div>
      </div>

      {mappings.map((mapping, index) => {
        const targetCol = targetColumns.find((col) => col.name === mapping.targetColumn);

        return (
          <div key={index} className="grid grid-cols-[1fr,auto,1fr] gap-4 items-center">
            <div className="p-2 bg-muted rounded font-mono text-sm">
              {mapping.sourceColumn}
            </div>

            <ArrowRight className="w-4 h-4 text-muted-foreground" />

            <Select
              value={mapping.targetColumn}
              onValueChange={(value) => handleMappingChange(mapping.sourceColumn, value)}
            >
              <SelectTrigger>
                <SelectValue placeholder="Skip column" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="">Skip this column</SelectItem>
                {targetColumns.map((col) => (
                  <SelectItem key={col.name} value={col.name}>
                    {col.name} ({col.data_type})
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        );
      })}

      <div className="pt-4 text-sm text-muted-foreground">
        <p>
          {mappings.filter((m) => m.targetColumn).length} of {mappings.length} columns mapped
        </p>
      </div>
    </div>
  );
}
