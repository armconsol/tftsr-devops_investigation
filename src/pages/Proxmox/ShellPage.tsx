import React from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Button } from '@/components/ui/index';
import { ArrowLeft } from 'lucide-react';
import { NodeShellConsole } from '@/components/Proxmox/NodeShellConsole';

/**
 * Dedicated in-app route for a host (node) shell.
 * Route: /proxmox/shell/:clusterId/:node
 */
export function ProxmoxShellPage() {
  const navigate = useNavigate();
  const params = useParams<{ clusterId: string; node: string }>();
  const clusterId = params.clusterId ?? '';
  const node = params.node ?? '';

  return (
    <div className="flex h-[calc(100vh-2rem)] flex-col gap-3 p-4">
      <div>
        <Button variant="ghost" size="sm" onClick={() => navigate(-1)}>
          <ArrowLeft className="mr-2 h-4 w-4" />
          Back
        </Button>
      </div>
      <div className="min-h-0 flex-1">
        <NodeShellConsole clusterId={clusterId} node={node} />
      </div>
    </div>
  );
}
