import { ReactNode } from "react";

export type WidgetSize = "1x1" | "1x2" | "2x1" | "2x2";

export interface WidgetPosition {
  x: number;
  y: number;
}

export interface WidgetConfig {
  id: string;
  type: string;
  title: string;
  size: WidgetSize;
  position: WidgetPosition;
  visible: boolean;
  refreshInterval?: number;
}

export interface WidgetData {
  loading: boolean;
  error?: string;
  data?: unknown;
  lastRefresh?: number;
}

export interface WidgetProps {
  id: string;
  title: string;
  size: WidgetSize;
  data: WidgetData;
  onRefresh?: () => void;
  onClose?: () => void;
  onResize?: (newSize: WidgetSize) => void;
  children?: ReactNode;
}

export interface DashboardLayoutProps {
  widgets: WidgetConfig[];
  onAddWidget: (type: string, title: string) => void;
  onRemoveWidget: (id: string) => void;
  onReorderWidget: (id: string, position: WidgetPosition) => void;
  onResizeWidget: (id: string, size: WidgetSize) => void;
  onToggleVisibility: (id: string, visible: boolean) => void;
  onRefreshWidget: (id: string) => void;
  children?: ReactNode;
}
