import { useEffect, useState } from 'react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { Button } from '@/components/ui';
import { Badge } from '@/components/ui';
import { AlertTriangle, Shield, Terminal, X } from 'lucide-react';
import { respondToShellApprovalCmd } from '@/lib/tauriCommands';

interface ShellApprovalRequest {
  approval_id: string;
  command: string;
  tier: number;
  reasoning: string;
  risk_factors: string[];
}

export function ShellApprovalModal() {
  const [request, setRequest] = useState<ShellApprovalRequest | null>(null);
  const [isResponding, setIsResponding] = useState(false);

  useEffect(() => {
    let unlisten: UnlistenFn;

    const setupListener = async () => {
      unlisten = await listen<ShellApprovalRequest>(
        'shell:approval-needed',
        (event) => {
          setRequest(event.payload);
        }
      );
    };

    setupListener();

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, []);

  const handleResponse = async (decision: string) => {
    if (!request) return;

    setIsResponding(true);
    try {
      await respondToShellApprovalCmd(request.approval_id, decision);
      setRequest(null);
    } catch (error) {
      console.error('Failed to respond to approval:', error);
    } finally {
      setIsResponding(false);
    }
  };

  const handleDeny = () => handleResponse('deny');
  const handleAllowOnce = () => handleResponse('allow_once');
  const handleAllowSession = () => handleResponse('allow_session');

  if (!request) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="relative bg-card border rounded-lg shadow-lg max-w-2xl w-full max-h-[90vh] overflow-y-auto m-4">
        {/* Header */}
        <div className="sticky top-0 bg-card border-b p-4 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Shield className="h-5 w-5 text-yellow-600" />
            <h2 className="text-lg font-semibold">Command Approval Required</h2>
          </div>
          <button
            onClick={() => !isResponding && setRequest(null)}
            disabled={isResponding}
            className="p-1 rounded hover:bg-accent text-muted-foreground"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        {/* Content */}
        <div className="p-6 space-y-4">
          <p className="text-sm text-muted-foreground">
            This command requires your approval before execution
          </p>

          {/* Command Display */}
          <div className="rounded-lg bg-slate-950 p-4 font-mono text-sm">
            <div className="flex items-center gap-2 mb-2">
              <Terminal className="h-4 w-4 text-slate-400" />
              <span className="text-slate-400">Command:</span>
            </div>
            <code className="text-green-400">{request.command}</code>
          </div>

          {/* Tier Badge */}
          <div className="flex items-center gap-2">
            <span className="text-sm text-muted-foreground">Safety Tier:</span>
            <Badge variant="outline" className="bg-yellow-50 text-yellow-700 border-yellow-300 dark:bg-yellow-900/40 dark:text-yellow-300 dark:border-yellow-700">
              Tier {request.tier}
            </Badge>
          </div>

          {/* Reasoning */}
          <div className="rounded-lg border border-yellow-300 bg-yellow-50 p-4">
            <div className="flex gap-2">
              <AlertTriangle className="h-4 w-4 text-yellow-700 flex-shrink-0 mt-0.5" />
              <div>
                <div className="font-medium text-yellow-900 mb-1">Why approval is needed:</div>
                <div className="text-sm text-yellow-800">{request.reasoning}</div>
              </div>
            </div>
          </div>

          {/* Risk Factors */}
          {request.risk_factors.length > 0 && (
            <div className="space-y-2">
              <div className="text-sm font-medium">Risk Factors:</div>
              <ul className="list-disc list-inside text-sm text-muted-foreground space-y-1">
                {request.risk_factors.map((factor, idx) => (
                  <li key={idx}>{factor}</li>
                ))}
              </ul>
            </div>
          )}

          {/* Safety Notice */}
          <div className="rounded-lg bg-muted p-3 text-sm text-muted-foreground">
            <div className="font-medium mb-1">Safety Controls:</div>
            <ul className="list-disc list-inside space-y-1 text-xs">
              <li>Command execution is logged and auditable</li>
              <li>30-second timeout protection</li>
              <li>PII detection before execution</li>
              <li>Output is captured for review</li>
            </ul>
          </div>
        </div>

        {/* Footer */}
        <div className="sticky bottom-0 bg-card border-t p-4 flex flex-col sm:flex-row gap-2 justify-end">
          <Button
            variant="destructive"
            onClick={handleDeny}
            disabled={isResponding}
            className="w-full sm:w-auto"
          >
            Deny
          </Button>
          <Button
            variant="outline"
            onClick={handleAllowOnce}
            disabled={isResponding}
            className="w-full sm:w-auto"
          >
            Allow Once
          </Button>
          <Button
            onClick={handleAllowSession}
            disabled={isResponding}
            className="w-full sm:w-auto"
          >
            Allow for Session
          </Button>
        </div>
      </div>
    </div>
  );
}
