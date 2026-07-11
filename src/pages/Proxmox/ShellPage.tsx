import React from 'react';
import { useParams, useNavigate, useSearchParams } from 'react-router-dom';
import { Button } from '@/components/ui/index';
import { ArrowLeft } from 'lucide-react';
import { NodeShellConsole } from '@/components/Proxmox/NodeShellConsole';

const ALLOWED_SHELL_CMDS = ['login', 'upgrade'] as const;
type ShellCmd = (typeof ALLOWED_SHELL_CMDS)[number];

function parseShellCmd(value: string | null): ShellCmd | undefined {
  return ALLOWED_SHELL_CMDS.includes(value as ShellCmd) ? (value as ShellCmd) : undefined;
}

/**
 * Dedicated in-app route for a host (node) shell.
 * Route: /proxmox/shell/:clusterId/:node?cmd=login|upgrade
 */
export function ProxmoxShellPage() {
  const navigate = useNavigate();
  const params = useParams<{ clusterId: string; node: string }>();
  const [searchParams] = useSearchParams();
  const clusterId = params.clusterId ?? '';
  const node = params.node ?? '';
  const cmd = parseShellCmd(searchParams.get('cmd'));

  return (
    <div className="flex h-[calc(100vh-2rem)] flex-col gap-3 p-4">
      <div>
        <Button variant="ghost" size="sm" onClick={() => navigate(-1)}>
          <ArrowLeft className="mr-2 h-4 w-4" />
          Back
        </Button>
      </div>
      <div className="min-h-0 flex-1">
        <NodeShellConsole clusterId={clusterId} node={node} cmd={cmd} />
      </div>
    </div>
  );
}
