import { useMemo } from "react";
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
  Filler,
  type ChartOptions,
} from "chart.js";
import { Line } from "react-chartjs-2";

// Register Chart.js components once at module load.
ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
  Filler
);

export type MetricsChartType = "cpu" | "memory";

export interface MetricsDataPoint {
  label: string;
  value: number;
}

export interface MetricsChartProps {
  /** Series of data points to render on the chart. */
  data: MetricsDataPoint[];
  /** Title displayed above the chart. */
  title: string;
  /** Whether this chart is showing CPU or Memory metrics. Used for label/color. */
  type: MetricsChartType;
  /** Optional fixed height in pixels. Defaults to 240. */
  height?: number;
}

const COLORS: Record<MetricsChartType, { border: string; background: string; label: string }> = {
  cpu: {
    border: "rgb(59, 130, 246)",
    background: "rgba(59, 130, 246, 0.2)",
    label: "CPU",
  },
  memory: {
    border: "rgb(16, 185, 129)",
    background: "rgba(16, 185, 129, 0.2)",
    label: "Memory",
  },
};

/**
 * Simple Chart.js line chart wrapper for displaying live pod/node metrics.
 *
 * Designed to be a thin wrapper around `react-chartjs-2`'s `Line` component
 * so callers can pass labelled values without re-implementing chart options.
 */
export function MetricsChart({ data, title, type, height = 240 }: MetricsChartProps) {
  const palette = COLORS[type];

  const chartData = useMemo(
    () => ({
      labels: data.map((d) => d.label),
      datasets: [
        {
          label: palette.label,
          data: data.map((d) => d.value),
          borderColor: palette.border,
          backgroundColor: palette.background,
          fill: true,
          tension: 0.3,
          pointRadius: 2,
        },
      ],
    }),
    [data, palette.border, palette.background, palette.label]
  );

  const options: ChartOptions<"line"> = useMemo(
    () => ({
      responsive: true,
      maintainAspectRatio: false,
      plugins: {
        legend: { display: true, position: "top" as const },
        title: { display: Boolean(title), text: title },
        tooltip: { intersect: false, mode: "index" as const },
      },
      scales: {
        x: { grid: { display: false } },
        y: { beginAtZero: true },
      },
      interaction: { mode: "index" as const, intersect: false },
    }),
    [title]
  );

  if (data.length === 0) {
    return (
      <div
        className="flex items-center justify-center text-sm text-muted-foreground border rounded-lg bg-card"
        style={{ height }}
      >
        No metrics data available
      </div>
    );
  }

  return (
    <div className="bg-card border rounded-lg p-3" style={{ height }}>
      <Line data={chartData} options={options} />
    </div>
  );
}

export default MetricsChart;
