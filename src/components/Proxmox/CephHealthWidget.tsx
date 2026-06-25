import React from 'react';
import { Card, CardContent, CardHeader, CardTitle, Button } from '@/components/ui/index';
import { Alert, AlertDescription } from '@/components/ui/index';
import { AlertCircle, CheckCircle, XCircle } from 'lucide-react';

interface CephHealthInfo {
  status: 'HEALTH_OK' | 'HEALTH_WARN' | 'HEALTH_ERR';
  summary: string;
  details: string[];
}

interface CephHealthWidgetProps {
  health: CephHealthInfo;
  onRefresh?: () => void;
  isLoading?: boolean;
}

export function CephHealthWidget({
  health,
  onRefresh,
  isLoading,
}: CephHealthWidgetProps) {
  // Defensive: a malformed/partial health payload must never crash the page
  // (an uncaught render error blanks the whole Ceph view).
  const status = health?.status ?? 'unknown';
  const summary = health?.summary ?? '';
  const details = Array.isArray(health?.details) ? health.details : [];

  const getStatusColor = () => {
    switch (status) {
      case 'HEALTH_OK':
        return 'text-green-500';
      case 'HEALTH_WARN':
        return 'text-yellow-500';
      case 'HEALTH_ERR':
        return 'text-red-500';
      default:
        return 'text-gray-500';
    }
  };

  const getStatusIcon = () => {
    switch (status) {
      case 'HEALTH_OK':
        return <CheckCircle className="h-12 w-12 text-green-500" />;
      case 'HEALTH_WARN':
        return <AlertCircle className="h-12 w-12 text-yellow-500" />;
      case 'HEALTH_ERR':
        return <XCircle className="h-12 w-12 text-red-500" />;
      default:
        return <AlertCircle className="h-12 w-12 text-gray-500" />;
    }
  };

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle>Ceph Health</CardTitle>
        <Button
          variant="ghost"
          size="sm"
          onClick={onRefresh}
          disabled={isLoading}
        >
          <span className={`h-4 w-4 ${isLoading ? 'animate-spin' : ''}`}>↻</span>
        </Button>
      </CardHeader>
      <CardContent>
        <div className="flex items-center space-x-4 mb-4">
          {getStatusIcon()}
          <div>
            <h3 className={`text-2xl font-bold ${getStatusColor()}`}>
              {status}
            </h3>
            <p className="text-sm text-muted-foreground">{summary}</p>
          </div>
        </div>
        {details.length > 0 && (
          <div className="space-y-2">
            {details.map((detail, index) => (
              <Alert key={index} variant={detail.includes('error') ? 'destructive' : 'default'}>
                <AlertDescription>{detail}</AlertDescription>
              </Alert>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
