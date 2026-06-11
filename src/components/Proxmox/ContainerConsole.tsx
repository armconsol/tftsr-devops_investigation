import React, { useState, useEffect, useRef } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/index';
import { Terminal } from 'lucide-react';

interface ContainerConsoleProps {
  remoteId: string;
  containerId: number;
  node: string;
  onClose?: () => void;
  onConnect?: () => void;
  onDisconnect?: () => void;
}

export function ContainerConsole({ containerId, node, onClose, onConnect, onDisconnect }: ContainerConsoleProps) {
  const [connected, setConnected] = useState(false);
  const [error, setError] = useState<string>('');
  const [isConnecting, setIsConnecting] = useState(false);
  const terminalRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (connected && terminalRef.current) {
      terminalRef.current.focus();
    }
  }, [connected]);

  const handleConnect = async () => {
    setIsConnecting(true);
    setError('');

    try {
      await new Promise((resolve) => {
        setTimeout(() => {
          setConnected(true);
          setIsConnecting(false);
          onConnect?.();
          resolve(true);
        }, 1000);
      });
    } catch {
      setError('Failed to connect to container console');
      setIsConnecting(false);
    }
  };

  const handleDisconnect = () => {
    setConnected(false);
    setError('');
    onDisconnect?.();
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape' && connected) {
      handleDisconnect();
    }
  };

  return (
    <Card className="h-full flex flex-col">
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="flex items-center gap-2">
          <Terminal className="h-5 w-5" />
          Container Console - {node} / CT {containerId}
        </CardTitle>
        <div className="flex space-x-2">
          {connected ? (
            <Button variant="outline" size="sm" onClick={handleDisconnect}>
              Disconnect
            </Button>
          ) : (
            <Button size="sm" onClick={handleConnect} disabled={isConnecting}>
              {isConnecting ? 'Connecting...' : 'Connect'}
            </Button>
          )}
          {onClose && (
            <Button variant="ghost" size="sm" onClick={onClose}>
              Close
            </Button>
          )}
        </div>
      </CardHeader>
      <CardContent className="flex-1 overflow-hidden relative">
        {!connected && !error && (
          <div className="absolute inset-0 flex flex-col items-center justify-center bg-background/50">
            <Terminal className="h-16 w-16 text-muted-foreground mb-4" />
            <p className="text-muted-foreground">Click "Connect" to open container console</p>
          </div>
        )}

        {error && (
          <Alert variant="destructive">
            <AlertTitle>Connection Error</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {connected && (
          <div
            ref={terminalRef}
            tabIndex={0}
            onKeyDown={handleKeyDown}
            className="h-full w-full bg-black font-mono text-green-500 p-4 overflow-auto outline-none"
            style={{ minHeight: '400px' }}
          >
            <div className="mb-2 text-sm text-gray-500">
              Container Console - Press ESC to disconnect
            </div>
            <div className="space-y-1">
              <div>Proxmox VE Container Console</div>
              <div>Connected to {node} / CT {containerId}</div>
              <div>----------------------------------------</div>
              <div className="animate-pulse">_</div>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
