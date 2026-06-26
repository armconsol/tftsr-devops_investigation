import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MetricsChart } from "@/components/Kubernetes/MetricsChart";

vi.mock("recharts", () => ({
  LineChart: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="line-chart">{children}</div>
  ),
  BarChart: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="bar-chart">{children}</div>
  ),
  Line: () => null,
  Bar: () => null,
  XAxis: () => null,
  YAxis: () => null,
  CartesianGrid: () => null,
  Tooltip: () => null,
  Legend: () => null,
  ResponsiveContainer: ({ children }: { children: React.ReactNode }) => (
    <div>{children}</div>
  ),
}));

const sampleData = {
  labels: ["00:00", "04:00", "08:00"],
  datasets: [
    {
      label: "CPU Usage",
      data: [12, 18, 22],
      borderColor: "hsl(var(--primary))",
    },
  ],
};

const emptyData = {
  labels: [],
  datasets: [],
};

describe("MetricsChart", () => {
  it("renders the title", () => {
    render(<MetricsChart title="CPU Metrics" data={sampleData} />);
    expect(screen.getByText("CPU Metrics")).toBeInTheDocument();
  });

  it("renders a line chart by default (type='line')", () => {
    render(<MetricsChart title="CPU Metrics" data={sampleData} />);
    expect(screen.getByTestId("line-chart")).toBeInTheDocument();
    expect(screen.queryByTestId("bar-chart")).not.toBeInTheDocument();
  });

  it("renders a bar chart when type='bar'", () => {
    render(<MetricsChart title="Memory Metrics" data={sampleData} type="bar" />);
    expect(screen.getByTestId("bar-chart")).toBeInTheDocument();
    expect(screen.queryByTestId("line-chart")).not.toBeInTheDocument();
  });

  it("time range selector shows correct options", () => {
    render(<MetricsChart title="CPU Metrics" data={sampleData} />);
    expect(screen.getByRole("button", { name: "5m" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "15m" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "1h" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "6h" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "1d" })).toBeInTheDocument();
  });

  it("active time range is highlighted", () => {
    render(<MetricsChart title="CPU Metrics" data={sampleData} defaultTimeRange="1h" />);
    const activeButton = screen.getByRole("button", { name: "1h" });
    expect(activeButton.className).toMatch(/bg-primary|bg-accent/);
  });

  it("handles empty data gracefully without crashing", () => {
    render(<MetricsChart title="No Data Chart" data={emptyData} />);
    expect(screen.getByText("No Data Chart")).toBeInTheDocument();
    expect(screen.getByText(/no metrics data/i)).toBeInTheDocument();
  });

  it("allows changing the active time range via buttons", async () => {
    const user = userEvent.setup();
    render(<MetricsChart title="CPU Metrics" data={sampleData} />);
    const button6h = screen.getByRole("button", { name: "6h" });
    await user.click(button6h);
    expect(button6h.className).toMatch(/bg-primary|bg-accent/);
  });
});
