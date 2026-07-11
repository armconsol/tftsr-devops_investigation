// Data Visualization Component — render query results as charts

import { useState, useMemo, useRef } from 'react';
import {
  BarChart,
  Bar,
  LineChart,
  Line,
  PieChart,
  Pie,
  Cell,
  ScatterChart,
  Scatter,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';
import { Button, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui';
import { Download, BarChart3, LineChart as LineIcon, PieChart as PieIcon, ScatterChart as ScatterIcon } from 'lucide-react';
import { toast } from 'sonner';
import type { QueryResult } from '@/lib/tauriCommands';

type ChartType = 'bar' | 'line' | 'pie' | 'scatter' | 'histogram';

interface DataVisualizationProps {
  result: QueryResult;
}

const CHART_COLORS = [
  '#3b82f6', '#ef4444', '#10b981', '#f59e0b', '#8b5cf6',
  '#ec4899', '#06b6d4', '#84cc16', '#f97316', '#6366f1',
];

export function DataVisualization({ result }: DataVisualizationProps) {
  const containerRef = useRef<HTMLDivElement>(null);

  const numericColumns = useMemo(
    () =>
      result.columns.filter((c) => {
        const t = c.data_type.toLowerCase();
        return (
          t.includes('int') ||
          t.includes('float') ||
          t.includes('double') ||
          t.includes('numeric') ||
          t.includes('decimal') ||
          t.includes('real') ||
          t.includes('serial') ||
          t.includes('number')
        );
      }),
    [result.columns]
  );

  const temporalColumns = useMemo(
    () =>
      result.columns.filter((c) => {
        const t = c.data_type.toLowerCase();
        return t.includes('date') || t.includes('time') || t.includes('timestamp');
      }),
    [result.columns]
  );

  const categoricalColumns = useMemo(
    () => result.columns.filter((c) => !numericColumns.includes(c)),
    [result.columns, numericColumns]
  );

  const [chartType, setChartType] = useState<ChartType>('bar');
  const [xColumn, setXColumn] = useState<string>(
    temporalColumns[0]?.name || categoricalColumns[0]?.name || result.columns[0]?.name || ''
  );
  const [yColumn, setYColumn] = useState<string>(numericColumns[0]?.name || '');

  const xIndex = result.columns.findIndex((c) => c.name === xColumn);
  const yIndex = result.columns.findIndex((c) => c.name === yColumn);

  const chartData = useMemo(() => {
    if (xIndex < 0 || yIndex < 0) return [];

    if (chartType === 'histogram') {
      // Group counts by x value
      const counts: Record<string, number> = {};
      for (const row of result.rows) {
        const key = String(row[xIndex] ?? 'NULL');
        counts[key] = (counts[key] || 0) + 1;
      }
      return Object.entries(counts).map(([name, value]) => ({ name, value }));
    }

    return result.rows.slice(0, 500).map((row) => ({
      name: String(row[xIndex] ?? 'NULL'),
      value: typeof row[yIndex] === 'number' ? row[yIndex] : parseFloat(String(row[yIndex] ?? 0)),
      x: row[xIndex],
      y: row[yIndex],
    }));
  }, [chartType, xIndex, yIndex, result.rows]);

  const exportAsPng = async () => {
    if (!containerRef.current) return;
    const svg = containerRef.current.querySelector('svg');
    if (!svg) {
      toast.error('No chart to export');
      return;
    }

    try {
      const svgData = new XMLSerializer().serializeToString(svg);
      const svgBlob = new Blob([svgData], { type: 'image/svg+xml;charset=utf-8' });
      const url = URL.createObjectURL(svgBlob);

      const img = new Image();
      const rect = svg.getBoundingClientRect();
      img.width = rect.width;
      img.height = rect.height;

      await new Promise<void>((resolve, reject) => {
        img.onload = () => {
          const canvas = document.createElement('canvas');
          canvas.width = rect.width * 2;
          canvas.height = rect.height * 2;
          const ctx = canvas.getContext('2d');
          if (!ctx) {
            reject(new Error('Failed to get canvas context'));
            return;
          }
          ctx.fillStyle = '#ffffff';
          ctx.fillRect(0, 0, canvas.width, canvas.height);
          ctx.scale(2, 2);
          ctx.drawImage(img, 0, 0);

          canvas.toBlob((blob) => {
            if (!blob) {
              reject(new Error('Failed to create blob'));
              return;
            }
            const link = document.createElement('a');
            link.download = `chart-${chartType}-${Date.now()}.png`;
            link.href = URL.createObjectURL(blob);
            link.click();
            URL.revokeObjectURL(link.href);
            URL.revokeObjectURL(url);
            resolve();
          }, 'image/png');
        };
        img.onerror = () => reject(new Error('Failed to load SVG'));
        img.src = url;
      });

      toast.success('Chart exported as PNG');
    } catch (error) {
      toast.error('Export failed: ' + String(error));
    }
  };

  if (result.rows.length === 0) {
    return (
      <div className="flex items-center justify-center h-64 text-muted-foreground">
        <p>No data to visualize.</p>
      </div>
    );
  }

  if (numericColumns.length === 0 && chartType !== 'histogram') {
    return (
      <div className="flex flex-col items-center justify-center h-64 text-muted-foreground gap-2">
        <p>No numeric columns detected.</p>
        <p className="text-xs">Only histogram chart available — switch chart type to visualize.</p>
        <Button onClick={() => setChartType('histogram')} size="sm" variant="outline">
          Switch to Histogram
        </Button>
      </div>
    );
  }

  const renderChart = () => {
    const margin = { top: 20, right: 30, left: 20, bottom: 60 };

    switch (chartType) {
      case 'bar':
        return (
          <BarChart data={chartData} margin={margin}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="name" angle={-45} textAnchor="end" height={80} />
            <YAxis />
            <Tooltip />
            <Legend />
            <Bar dataKey="value" fill={CHART_COLORS[0]} name={yColumn} />
          </BarChart>
        );
      case 'line':
        return (
          <LineChart data={chartData} margin={margin}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="name" angle={-45} textAnchor="end" height={80} />
            <YAxis />
            <Tooltip />
            <Legend />
            <Line type="monotone" dataKey="value" stroke={CHART_COLORS[0]} name={yColumn} />
          </LineChart>
        );
      case 'pie':
        return (
          <PieChart margin={margin}>
            <Pie
              data={chartData}
              dataKey="value"
              nameKey="name"
              cx="50%"
              cy="50%"
              outerRadius={120}
              label
            >
              {chartData.map((_, index) => (
                <Cell key={`cell-${index}`} fill={CHART_COLORS[index % CHART_COLORS.length]} />
              ))}
            </Pie>
            <Tooltip />
            <Legend />
          </PieChart>
        );
      case 'scatter':
        return (
          <ScatterChart margin={margin}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="x" name={xColumn} />
            <YAxis dataKey="y" name={yColumn} />
            <Tooltip cursor={{ strokeDasharray: '3 3' }} />
            <Legend />
            <Scatter name={`${xColumn} vs ${yColumn}`} data={chartData} fill={CHART_COLORS[0]} />
          </ScatterChart>
        );
      case 'histogram':
        return (
          <BarChart data={chartData} margin={margin}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="name" angle={-45} textAnchor="end" height={80} />
            <YAxis />
            <Tooltip />
            <Legend />
            <Bar dataKey="value" fill={CHART_COLORS[1]} name="Count" />
          </BarChart>
        );
    }
  };

  return (
    <div className="flex flex-col h-full p-4">
      <div className="flex flex-wrap items-center gap-3 mb-4">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium">Chart:</span>
          <Select value={chartType} onValueChange={(v) => setChartType(v as ChartType)}>
            <SelectTrigger className="w-36">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="bar">
                <span className="flex items-center gap-2"><BarChart3 className="w-4 h-4" />Bar</span>
              </SelectItem>
              <SelectItem value="line">
                <span className="flex items-center gap-2"><LineIcon className="w-4 h-4" />Line</span>
              </SelectItem>
              <SelectItem value="pie">
                <span className="flex items-center gap-2"><PieIcon className="w-4 h-4" />Pie</span>
              </SelectItem>
              <SelectItem value="scatter">
                <span className="flex items-center gap-2"><ScatterIcon className="w-4 h-4" />Scatter</span>
              </SelectItem>
              <SelectItem value="histogram">
                <span className="flex items-center gap-2"><BarChart3 className="w-4 h-4" />Histogram</span>
              </SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div className="flex items-center gap-2">
          <span className="text-sm font-medium">X:</span>
          <Select value={xColumn} onValueChange={setXColumn}>
            <SelectTrigger className="w-40">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {result.columns.map((c) => (
                <SelectItem key={c.name} value={c.name}>
                  {c.name} ({c.data_type})
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        {chartType !== 'histogram' && (
          <div className="flex items-center gap-2">
            <span className="text-sm font-medium">Y:</span>
            <Select value={yColumn} onValueChange={setYColumn}>
              <SelectTrigger className="w-40">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {numericColumns.map((c) => (
                  <SelectItem key={c.name} value={c.name}>
                    {c.name} ({c.data_type})
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        )}

        <div className="ml-auto">
          <Button onClick={exportAsPng} variant="outline" size="sm">
            <Download className="w-4 h-4 mr-2" />
            Export PNG
          </Button>
        </div>
      </div>

      <div ref={containerRef} className="flex-1 min-h-[300px]">
        <ResponsiveContainer width="100%" height="100%">
          {renderChart()}
        </ResponsiveContainer>
      </div>
    </div>
  );
}
