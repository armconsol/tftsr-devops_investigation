import React from 'react';
import { Card, CardContent, CardHeader, Button } from '@/components/ui/index';
import { RefreshCw, X } from 'lucide-react';

interface WidgetContainerProps {
  title: string;
  children: React.ReactNode;
  onRefresh?: () => void;
  onClose?: () => void;
  isLoading?: boolean;
  size?: 'small' | 'medium' | 'large';
  className?: string;
}

export function WidgetContainer({
  title,
  children,
  onRefresh,
  onClose,
  isLoading,
  size = 'medium',
  className = '',
}: WidgetContainerProps) {
  const sizeClasses = {
    small: 'h-48',
    medium: 'h-64',
    large: 'h-80',
  };

  return (
    <Card className={`flex flex-col ${sizeClasses[size]} ${className}`}>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <h3 className="text-sm font-medium">{title}</h3>
        <div className="flex space-x-1">
          {onRefresh && (
            <Button
              variant="ghost"
              size="sm"
              onClick={onRefresh}
              disabled={isLoading}
            >
              <RefreshCw className={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`} />
            </Button>
          )}
          {onClose && (
            <Button variant="ghost" size="sm" onClick={onClose}>
              <X className="h-4 w-4" />
            </Button>
          )}
        </div>
      </CardHeader>
      <CardContent className="flex-1 overflow-auto">{children}</CardContent>
    </Card>
  );
}
