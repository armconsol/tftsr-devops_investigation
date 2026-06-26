import React, { useState } from 'react';
import { WidgetContainer } from './WidgetContainer';

interface WidgetConfig {
  id: string;
  type: string;
  title: string;
  size: 'small' | 'medium' | 'large';
  position: { x: number; y: number };
  visible: boolean;
}

interface DashboardLayoutProps {
  widgets: WidgetConfig[];
  onWidgetUpdate?: (widget: WidgetConfig) => void;
  onWidgetRemove?: (id: string) => void;
  onWidgetRefresh?: (id: string) => void;
  className?: string;
}

export function DashboardLayout({
  widgets,
  onWidgetRemove,
  onWidgetRefresh,
  className = '',
}: DashboardLayoutProps) {
  const [layout, setLayout] = useState<WidgetConfig[]>(widgets);

  const handleRefresh = (id: string) => {
    onWidgetRefresh?.(id);
  };

  const handleClose = (id: string) => {
    onWidgetRemove?.(id);
    setLayout((prev) => prev.filter((w) => w.id !== id));
  };

  return (
    <div className={`grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 ${className}`}>
      {layout
        .filter((widget) => widget.visible)
        .map((widget) => (
          <WidgetContainer
            key={widget.id}
            title={widget.title}
            onClose={() => handleClose(widget.id)}
            onRefresh={() => handleRefresh(widget.id)}
            size={widget.size}
          >
            {/* Widget content will be rendered by parent */}
            <div className="text-center text-muted-foreground">
              Widget {widget.type} content
            </div>
          </WidgetContainer>
        ))}
    </div>
  );
}
