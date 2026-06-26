import React from 'react';
import { WidgetContainer } from './WidgetContainer';
import { Card, CardContent, CardHeader } from '@/components/ui/index';
import { AlertCircle, CheckCircle, XCircle } from 'lucide-react';

interface SubscriptionInfo {
  id: string;
  cluster: string;
  status: 'active' | 'mixed' | 'none' | 'unknown';
  level: string;
  socket: number;
  expiry?: string;
  keyId?: string;
}

interface SubscriptionWidgetProps {
  subscriptions: SubscriptionInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
}

export function SubscriptionWidget({
  subscriptions,
  onRefresh,
  isLoading,
}: SubscriptionWidgetProps) {
  const activeCount = subscriptions.filter((s) => s.status === 'active').length;
  const noneCount = subscriptions.filter((s) => s.status === 'none').length;
  const unknownCount = subscriptions.filter((s) => s.status === 'unknown').length;

  return (
    <WidgetContainer
      title="Subscriptions"
      onRefresh={onRefresh}
      isLoading={isLoading}
      size="medium"
    >
      <div className="grid grid-cols-3 gap-4 mb-4">
        <div className="text-center">
          <div className="text-2xl font-bold">{activeCount}</div>
          <div className="text-xs text-muted-foreground">Active</div>
        </div>
        <div className="text-center">
          <div className="text-2xl font-bold">{noneCount}</div>
          <div className="text-xs text-muted-foreground">None</div>
        </div>
        <div className="text-center">
          <div className="text-2xl font-bold">{unknownCount}</div>
          <div className="text-xs text-muted-foreground">Unknown</div>
        </div>
      </div>
      <div className="space-y-2">
        {subscriptions.map((sub) => (
          <Card key={sub.id}>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <div className="flex items-center space-x-2">
                {sub.status === 'active' ? (
                  <CheckCircle className="h-4 w-4 text-green-500" />
                ) : sub.status === 'none' ? (
                  <XCircle className="h-4 w-4 text-red-500" />
                ) : (
                  <AlertCircle className="h-4 w-4 text-yellow-500" />
                )}
                <span className="font-medium">{sub.cluster}</span>
              </div>
              <span className="text-xs text-muted-foreground capitalize">
                {sub.status}
              </span>
            </CardHeader>
            <CardContent className="space-y-2">
              <div className="flex justify-between text-xs">
                <span>Level</span>
                <span>{sub.level}</span>
              </div>
              <div className="flex justify-between text-xs">
                <span>Socket</span>
                <span>{sub.socket}</span>
              </div>
              {sub.expiry && (
                <div className="flex justify-between text-xs">
                  <span>Expiry</span>
                  <span>{sub.expiry}</span>
                </div>
              )}
              {sub.keyId && (
                <div className="flex justify-between text-xs">
                  <span>Key ID</span>
                  <span>{sub.keyId}</span>
                </div>
              )}
            </CardContent>
          </Card>
        ))}
      </div>
    </WidgetContainer>
  );
}
