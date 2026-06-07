import React from "react";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui";

interface MetricsChartProps {
  title: string;
  data: { labels: string[]; datasets: { label: string; data: number[]; borderColor?: string; backgroundColor?: string }[] };
  type?: "line" | "bar";
  timeRange?: string;
  onTimeRangeChange?: (range: string) => void;
}

export function MetricsChart({ title, data, timeRange = "5m", onTimeRangeChange }: MetricsChartProps) {
  const timeRanges = ["5m", "15m", "1h", "6h", "1d", "7d"];

  return (
    <Card className="h-full flex flex-col">
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2">{title}</CardTitle>
          {onTimeRangeChange && (
            <div className="flex items-center gap-2">
              <span className="text-sm text-muted-foreground">Time Range:</span>
              <Select value={timeRange} onValueChange={onTimeRangeChange}>
                <SelectTrigger className="w-[120px]">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {timeRanges.map((range) => (
                    <SelectItem key={range} value={range}>
                      {range}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          )}
        </div>
      </CardHeader>
      <CardContent className="flex-1 min-h-[300px] flex items-center justify-center">
        {data.datasets.length > 0 ? (
          <div className="text-center">
            <p className="text-sm text-muted-foreground">Chart visualization would be displayed here</p>
            <p className="text-xs mt-2">Charts require react-chartjs-2 and chart.js dependencies</p>
          </div>
        ) : (
          <div className="text-center text-muted-foreground">
            No metrics data available
          </div>
        )}
      </CardContent>
    </Card>
  );
}
