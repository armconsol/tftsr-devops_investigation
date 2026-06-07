import React, { useState } from "react";
import {
  LineChart,
  BarChart,
  Line,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from "recharts";

const TIME_RANGES = ["5m", "15m", "1h", "6h", "1d"] as const;
type TimeRange = (typeof TIME_RANGES)[number];

interface ChartDataset {
  label: string;
  data: number[];
  borderColor?: string;
  backgroundColor?: string;
}

interface ChartData {
  labels: string[];
  datasets: ChartDataset[];
}

interface MetricsChartProps {
  title: string;
  data: ChartData;
  type?: "line" | "bar";
  height?: number;
  defaultTimeRange?: TimeRange;
}

const COLORS = [
  "hsl(var(--primary))",
  "#10b981",
  "#f59e0b",
  "#ef4444",
  "#8b5cf6",
  "#06b6d4",
];

function buildRechartsData(data: ChartData): Record<string, unknown>[] {
  return data.labels.map((label, i) => {
    const point: Record<string, unknown> = { name: label };
    for (const dataset of data.datasets) {
      point[dataset.label] = dataset.data[i] ?? null;
    }
    return point;
  });
}

export function MetricsChart({
  title,
  data,
  type = "line",
  height = 300,
  defaultTimeRange = "5m",
}: MetricsChartProps) {
  const [activeRange, setActiveRange] = useState<TimeRange>(
    (TIME_RANGES.includes(defaultTimeRange as TimeRange) ? defaultTimeRange : "5m") as TimeRange
  );

  const chartData = buildRechartsData(data);
  const hasData = data.datasets.length > 0 && data.labels.length > 0;

  return (
    <div className="bg-card rounded-lg border flex flex-col h-full">
      <div className="flex items-center justify-between px-4 py-3 border-b">
        <h3 className="font-semibold text-sm">{title}</h3>
        <div className="flex items-center gap-1">
          {TIME_RANGES.map((range) => (
            <button
              key={range}
              role="button"
              aria-label={range}
              onClick={() => setActiveRange(range)}
              className={[
                "px-2 py-0.5 rounded text-xs font-medium transition-colors",
                activeRange === range
                  ? "bg-primary text-primary-foreground"
                  : "text-muted-foreground hover:text-foreground hover:bg-accent",
              ].join(" ")}
            >
              {range}
            </button>
          ))}
        </div>
      </div>

      <div className="flex-1 p-4" style={{ minHeight: height }}>
        {!hasData ? (
          <div className="h-full flex items-center justify-center text-muted-foreground text-sm">
            No metrics data available
          </div>
        ) : (
          <ResponsiveContainer width="100%" height={height}>
            {type === "bar" ? (
              <BarChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" className="opacity-30" />
                <XAxis dataKey="name" tick={{ fontSize: 11 }} />
                <YAxis tick={{ fontSize: 11 }} />
                <Tooltip />
                <Legend />
                {data.datasets.map((dataset, idx) => (
                  <Bar
                    key={dataset.label}
                    dataKey={dataset.label}
                    fill={dataset.backgroundColor ?? COLORS[idx % COLORS.length]}
                  />
                ))}
              </BarChart>
            ) : (
              <LineChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" className="opacity-30" />
                <XAxis dataKey="name" tick={{ fontSize: 11 }} />
                <YAxis tick={{ fontSize: 11 }} />
                <Tooltip />
                <Legend />
                {data.datasets.map((dataset, idx) => (
                  <Line
                    key={dataset.label}
                    type="monotone"
                    dataKey={dataset.label}
                    stroke={dataset.borderColor ?? COLORS[idx % COLORS.length]}
                    dot={false}
                    strokeWidth={2}
                  />
                ))}
              </LineChart>
            )}
          </ResponsiveContainer>
        )}
      </div>
    </div>
  );
}
