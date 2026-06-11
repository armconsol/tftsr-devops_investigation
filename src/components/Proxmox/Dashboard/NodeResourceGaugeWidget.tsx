import React from 'react';
import { WidgetContainer } from './WidgetContainer';

interface ResourceGauge {
  label: string;
  value: number;
  max: number;
  color: string;
}

interface NodeResourceGaugeWidgetProps {
  cpu: ResourceGauge;
  memory: ResourceGauge;
  storage: ResourceGauge;
  onRefresh?: () => void;
  isLoading?: boolean;
}

function GaugeBar({
  label,
  value,
  max,
  color,
}: {
  label: string;
  value: number;
  max: number;
  color: string;
}) {
  const percentage = Math.min((value / max) * 100, 100);

  return (
    <div className="space-y-2">
      <div className="flex justify-between text-xs">
        <span className="font-medium">{label}</span>
        <span className="font-bold">{percentage.toFixed(1)}%</span>
      </div>
      <div className="h-2 w-full bg-slate-200 rounded-full overflow-hidden">
        <div
          className={`h-full rounded-full ${color}`}
          style={{ width: `${percentage}%` }}
        />
      </div>
      <div className="flex justify-between text-[10px] text-muted-foreground">
        <span>{value.toFixed(1)}</span>
        <span>{max.toFixed(1)}</span>
      </div>
    </div>
  );
}

export function NodeResourceGaugeWidget({
  cpu,
  memory,
  storage,
  onRefresh,
  isLoading,
}: NodeResourceGaugeWidgetProps) {
  return (
    <WidgetContainer
      title="Resource Gauges"
      onRefresh={onRefresh}
      isLoading={isLoading}
      size="small"
    >
      <div className="space-y-4">
        <GaugeBar
          label="CPU"
          value={cpu.value}
          max={cpu.max}
          color={cpu.color}
        />
        <GaugeBar
          label="Memory"
          value={memory.value}
          max={memory.max}
          color={memory.color}
        />
        <GaugeBar
          label="Storage"
          value={storage.value}
          max={storage.max}
          color={storage.color}
        />
      </div>
    </WidgetContainer>
  );
}
