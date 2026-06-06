import { useState, useEffect } from 'react';
import { Terminal, CheckCircle, XCircle, Shield, History } from 'lucide-react';
import { Button, Card, CardHeader, CardTitle, CardContent, Badge } from '@/components/ui';
import { Link } from 'react-router-dom';
import {
  checkKubectlInstalledCmd,
  listCommandExecutionsCmd,
  type KubectlStatus,
  type CommandExecution,
} from '@/lib/tauriCommands';

export default function ShellExecution() {
  const [kubectlStatus, setKubectlStatus] = useState<KubectlStatus | null>(null);
  const [executions, setExecutions] = useState<CommandExecution[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');

  const loadKubectlStatus = async () => {
    try {
      const status = await checkKubectlInstalledCmd();
      setKubectlStatus(status);
    } catch (err) {
      setError(String(err));
    }
  };

  const loadExecutions = async () => {
    setIsLoading(true);
    try {
      const data = await listCommandExecutionsCmd();
      setExecutions(data);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    loadKubectlStatus();
    loadExecutions();
  }, []);

  const getTierBadge = (tier: number) => {
    const colors = {
      1: 'bg-green-100 text-green-700 border-green-300',
      2: 'bg-yellow-100 text-yellow-700 border-yellow-300',
      3: 'bg-red-100 text-red-700 border-red-300',
    };
    return colors[tier as keyof typeof colors] || colors[1];
  };

  const getStatusBadge = (status: string) => {
    const config = {
      auto: { label: 'Auto-executed', color: 'bg-blue-100 text-blue-700 border-blue-300' },
      approved: { label: 'Approved', color: 'bg-green-100 text-green-700 border-green-300' },
      denied: { label: 'Denied', color: 'bg-red-100 text-red-700 border-red-300' },
    };
    const statusConfig = config[status as keyof typeof config] || config.auto;
    return statusConfig;
  };

  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-3xl font-bold mb-2">Shell Execution</h1>
        <p className="text-muted-foreground">
          Configure and monitor autonomous shell command execution with intelligent safety controls
        </p>
      </div>

      {error && (
        <div className="rounded-lg border border-red-300 bg-red-50 p-4 text-sm text-red-800">
          {error}
        </div>
      )}

      {/* kubectl Status */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Terminal className="h-5 w-5" />
            kubectl Status
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {kubectlStatus ? (
            <>
              <div className="flex items-center gap-3">
                {kubectlStatus.installed ? (
                  <>
                    <CheckCircle className="h-5 w-5 text-green-600" />
                    <span className="font-medium text-green-700">kubectl is installed</span>
                  </>
                ) : (
                  <>
                    <XCircle className="h-5 w-5 text-red-600" />
                    <span className="font-medium text-red-700">kubectl is not installed</span>
                  </>
                )}
              </div>

              {kubectlStatus.path && (
                <div className="text-sm text-muted-foreground">
                  <span className="font-medium">Path:</span> {kubectlStatus.path}
                </div>
              )}

              {kubectlStatus.version && (
                <div className="rounded-lg bg-slate-950 p-3 font-mono text-xs text-slate-400 overflow-x-auto">
                  <pre>{kubectlStatus.version}</pre>
                </div>
              )}
            </>
          ) : (
            <p className="text-sm text-muted-foreground">Checking kubectl status...</p>
          )}

          <div className="pt-2">
            <Link to="/settings/kubeconfig">
              <Button variant="outline" className="w-full">
                Manage Kubeconfig Files
              </Button>
            </Link>
          </div>
        </CardContent>
      </Card>

      {/* Safety Architecture */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            Safety Architecture
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <p className="text-sm text-muted-foreground">
            Commands are automatically classified into three safety tiers:
          </p>

          <div className="space-y-3">
            <div className="flex items-start gap-3 p-3 rounded-lg bg-green-50 border border-green-200">
              <Badge className={getTierBadge(1)}>Tier 1</Badge>
              <div className="space-y-1">
                <div className="font-medium text-green-900">Auto-execute (Read-only)</div>
                <div className="text-sm text-green-800">
                  kubectl get, describe, logs | cat, grep, ls
                </div>
              </div>
            </div>

            <div className="flex items-start gap-3 p-3 rounded-lg bg-yellow-50 border border-yellow-200">
              <Badge className={getTierBadge(2)}>Tier 2</Badge>
              <div className="space-y-1">
                <div className="font-medium text-yellow-900">Require approval (Mutating)</div>
                <div className="text-sm text-yellow-800">
                  kubectl apply, delete, scale | ssh, chmod, systemctl restart
                </div>
              </div>
            </div>

            <div className="flex items-start gap-3 p-3 rounded-lg bg-red-50 border border-red-200">
              <Badge className={getTierBadge(3)}>Tier 3</Badge>
              <div className="space-y-1">
                <div className="font-medium text-red-900">Always deny (Destructive)</div>
                <div className="text-sm text-red-800">
                  rm -rf, shutdown, mkfs, dd
                </div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Command Execution History */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <History className="h-5 w-5" />
            Recent Command Executions ({executions.length})
          </CardTitle>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <p className="text-sm text-muted-foreground text-center py-8">Loading...</p>
          ) : executions.length === 0 ? (
            <p className="text-sm text-muted-foreground text-center py-8">
              No command executions yet
            </p>
          ) : (
            <div className="space-y-3">
              {executions.slice(0, 10).map((exec) => {
                const statusConfig = getStatusBadge(exec.approval_status);
                return (
                  <div key={exec.id} className="p-3 rounded-lg border space-y-2">
                    <div className="flex items-start justify-between">
                      <div className="flex-1 min-w-0">
                        <code className="text-sm font-mono text-foreground break-all">
                          {exec.command}
                        </code>
                      </div>
                      <div className="flex gap-2 ml-3 flex-shrink-0">
                        <Badge className={getTierBadge(exec.tier)}>
                          T{exec.tier}
                        </Badge>
                        <Badge className={statusConfig.color}>
                          {statusConfig.label}
                        </Badge>
                      </div>
                    </div>

                    <div className="flex items-center gap-4 text-xs text-muted-foreground">
                      {exec.exit_code !== undefined && (
                        <span className={exec.exit_code === 0 ? 'text-green-600' : 'text-red-600'}>
                          Exit: {exec.exit_code}
                        </span>
                      )}
                      {exec.execution_time_ms !== undefined && (
                        <span>{exec.execution_time_ms}ms</span>
                      )}
                      <span>{new Date(exec.executed_at).toLocaleString()}</span>
                    </div>

                    {exec.stdout && (
                      <details className="text-xs">
                        <summary className="cursor-pointer text-muted-foreground hover:text-foreground">
                          Show output
                        </summary>
                        <pre className="mt-2 p-2 rounded bg-slate-950 text-slate-400 overflow-x-auto max-h-40">
                          {exec.stdout}
                        </pre>
                      </details>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
