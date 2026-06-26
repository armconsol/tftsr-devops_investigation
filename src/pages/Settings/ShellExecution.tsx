import { useState, useEffect } from 'react';
import { Terminal, CheckCircle, XCircle, Shield, History, ChevronDown } from 'lucide-react';
import { Button, Card, CardHeader, CardTitle, CardContent, Badge } from '@/components/ui';
import { Link } from 'react-router-dom';
import {
  checkKubectlInstalledCmd,
  listCommandExecutionsCmd,
  getClassifierRulesCmd,
  type KubectlStatus,
  type CommandExecution,
  type ClassifierRules,
} from '@/lib/tauriCommands';

// ── Tier display config ───────────────────────────────────────────────────────

interface TierConfig {
  label: string;
  behavior: string;
  colorBg: string;
  colorBorder: string;
  colorHeading: string;
  colorText: string;
  badgeClass: string;
  tier: 1 | 2 | 3;
}

const TIER_CONFIG: TierConfig[] = [
  {
    tier: 1,
    label: 'Tier 1',
    behavior: 'Auto-execute (Read-only)',
    colorBg: 'bg-green-50 dark:bg-green-950/30',
    colorBorder: 'border-green-200 dark:border-green-800',
    colorHeading: 'text-green-900 dark:text-green-300',
    colorText: 'text-green-800 dark:text-green-400',
    badgeClass: 'bg-green-100 text-green-700 border-green-300 dark:bg-green-900/40 dark:text-green-300 dark:border-green-700',
  },
  {
    tier: 2,
    label: 'Tier 2',
    behavior: 'Require approval (Mutating)',
    colorBg: 'bg-yellow-50 dark:bg-yellow-950/30',
    colorBorder: 'border-yellow-200 dark:border-yellow-800',
    colorHeading: 'text-yellow-900 dark:text-yellow-300',
    colorText: 'text-yellow-800 dark:text-yellow-400',
    badgeClass: 'bg-yellow-100 text-yellow-700 border-yellow-300 dark:bg-yellow-900/40 dark:text-yellow-300 dark:border-yellow-700',
  },
  {
    tier: 3,
    label: 'Tier 3',
    behavior: 'Always deny (Destructive)',
    colorBg: 'bg-red-50 dark:bg-red-950/30',
    colorBorder: 'border-red-200 dark:border-red-800',
    colorHeading: 'text-red-900 dark:text-red-300',
    colorText: 'text-red-800 dark:text-red-400',
    badgeClass: 'bg-red-100 text-red-700 border-red-300 dark:bg-red-900/40 dark:text-red-300 dark:border-red-700',
  },
];

// ── Helper: build per-tier category groups from ClassifierRules ───────────────

interface CategoryGroup {
  label: string;
  commands: string[];
}

function buildTier1Groups(rules: ClassifierRules): CategoryGroup[] {
  return [
    { label: 'kubectl', commands: rules.tier1_kubectl.map((c) => `kubectl ${c}`) },
    { label: 'systemctl', commands: rules.tier1_systemctl.map((c) => `systemctl ${c}`) },
    { label: 'proxmox', commands: rules.tier1_proxmox.map((c) => `<cmd> ${c}`) },
    { label: 'general', commands: rules.tier1_general },
  ].filter((g) => g.commands.length > 0);
}

function buildTier2Groups(rules: ClassifierRules): CategoryGroup[] {
  return [
    { label: 'kubectl', commands: rules.tier2_kubectl.map((c) => `kubectl ${c}`) },
    { label: 'systemctl', commands: rules.tier2_systemctl.map((c) => `systemctl ${c}`) },
    { label: 'proxmox', commands: rules.tier2_proxmox.map((c) => `<cmd> ${c}`) },
    { label: 'general', commands: rules.tier2_general },
  ].filter((g) => g.commands.length > 0);
}

function buildTier3Groups(rules: ClassifierRules): CategoryGroup[] {
  return [{ label: 'all', commands: rules.tier3 }];
}

const PREVIEW_COUNT = 6;

// ── Sub-components ────────────────────────────────────────────────────────────

function CommandChip({ cmd, colorText }: { cmd: string; colorText: string }) {
  return (
    <code
      className={`inline-block rounded px-1.5 py-0.5 text-xs font-mono border border-current/20 ${colorText}`}
    >
      {cmd}
    </code>
  );
}

interface TierCardProps {
  config: TierConfig;
  groups: CategoryGroup[];
}

function TierCard({ config, groups }: TierCardProps) {
  const [expanded, setExpanded] = useState(false);

  const allCommands = groups.flatMap((g) => g.commands);
  const total = allCommands.length;
  const previewCommands = allCommands.slice(0, PREVIEW_COUNT);
  const hasMore = total > PREVIEW_COUNT;

  return (
    <div
      className={`rounded-lg p-3 border ${config.colorBg} ${config.colorBorder}`}
      data-testid={`tier${config.tier}-card`}
    >
      <div className="flex items-start justify-between gap-2">
        <div className="flex items-start gap-3 min-w-0">
          <Badge className={config.badgeClass}>{config.label}</Badge>
          <div className="min-w-0">
            <div className={`font-medium ${config.colorHeading}`}>{config.behavior}</div>
            <div
              className={`mt-1.5 flex flex-wrap gap-1 ${config.colorText}`}
              data-testid={`tier${config.tier}-commands`}
            >
              {(expanded ? allCommands : previewCommands).map((cmd) => (
                <CommandChip key={cmd} cmd={cmd} colorText={config.colorText} />
              ))}
            </div>
          </div>
        </div>
        <span className={`shrink-0 text-xs font-mono tabular-nums ${config.colorText} opacity-70`}>
          {total}
        </span>
      </div>

      {hasMore && (
        <button
          onClick={() => setExpanded((p) => !p)}
          className={`mt-2 flex items-center gap-1 text-xs ${config.colorText} hover:opacity-80 transition-opacity`}
          data-testid={`tier${config.tier}-toggle`}
        >
          <ChevronDown
            className={`h-3 w-3 transition-transform ${expanded ? 'rotate-180' : ''}`}
          />
          {expanded ? 'Show fewer' : `Show all ${total} commands`}
        </button>
      )}
    </div>
  );
}

// ── Main component ────────────────────────────────────────────────────────────

export default function ShellExecution() {
  const [kubectlStatus, setKubectlStatus] = useState<KubectlStatus | null>(null);
  const [executions, setExecutions] = useState<CommandExecution[]>([]);
  const [classifierRules, setClassifierRules] = useState<ClassifierRules | null>(null);
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

  const loadClassifierRules = async () => {
    try {
      const rules = await getClassifierRulesCmd();
      setClassifierRules(rules);
    } catch {
      // Non-fatal — fall back to empty state; the tier cards will just show 0 commands
    }
  };

  useEffect(() => {
    loadKubectlStatus();
    loadExecutions();
    loadClassifierRules();
  }, []);

  const getTierBadge = (tier: number) => {
    const colors: Record<number, string> = {
      1: 'bg-green-100 text-green-700 border-green-300',
      2: 'bg-yellow-100 text-yellow-700 border-yellow-300',
      3: 'bg-red-100 text-red-700 border-red-300',
    };
    return colors[tier] ?? colors[1];
  };

  const getStatusBadge = (status: string) => {
    const config: Record<string, { label: string; color: string }> = {
      auto: { label: 'Auto-executed', color: 'bg-blue-100 text-blue-700 border-blue-300' },
      approved: { label: 'Approved', color: 'bg-green-100 text-green-700 border-green-300' },
      denied: { label: 'Denied', color: 'bg-red-100 text-red-700 border-red-300' },
    };
    return config[status] ?? config.auto;
  };

  // Build grouped command lists for each tier (empty arrays when rules not loaded)
  const tier1Groups = classifierRules ? buildTier1Groups(classifierRules) : [];
  const tier2Groups = classifierRules ? buildTier2Groups(classifierRules) : [];
  const tier3Groups = classifierRules ? buildTier3Groups(classifierRules) : [];

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

      {/* Safety Architecture — driven by live classifier data */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            Safety Architecture
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <p className="text-sm text-muted-foreground">
            Commands are automatically classified into three safety tiers. The lists below
            reflect the active classifier rules — they update whenever a rule is added or removed.
          </p>

          {!classifierRules && (
            <p className="text-xs text-muted-foreground">Loading classifier rules…</p>
          )}

          {TIER_CONFIG.map((cfg) => {
            const groups =
              cfg.tier === 1 ? tier1Groups : cfg.tier === 2 ? tier2Groups : tier3Groups;
            return <TierCard key={cfg.tier} config={cfg} groups={groups} />;
          })}
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
                        <Badge className={getTierBadge(exec.tier)}>T{exec.tier}</Badge>
                        <Badge className={statusConfig.color}>{statusConfig.label}</Badge>
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
