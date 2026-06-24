import React from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Button } from '@/components/ui/index';
import { ArrowLeft } from 'lucide-react';
import { NoVncConsole, type ConsoleKind } from '@/components/Proxmox/NoVncConsole';

/**
 * Dedicated in-app route for the noVNC graphical console.
 * Route: /proxmox/console/:clusterId/:node/:vmid/:kind
 */
export function ProxmoxConsolePage() {
  const navigate = useNavigate();
  const params = useParams<{ clusterId: string; node: string; vmid: string; kind: string }>();
  const clusterId = params.clusterId ?? '';
  const node = params.node ?? '';
  const vmId = Number(params.vmid ?? 0);
  const kind: ConsoleKind = params.kind === 'lxc' ? 'lxc' : 'qemu';

  return (
    <div className="flex h-[calc(100vh-2rem)] flex-col gap-3 p-4">
      <div>
        <Button variant="ghost" size="sm" onClick={() => navigate(-1)}>
          <ArrowLeft className="mr-2 h-4 w-4" />
          Back
        </Button>
      </div>
      <div className="min-h-0 flex-1">
        <NoVncConsole clusterId={clusterId} node={node} vmId={vmId} kind={kind} />
      </div>
    </div>
  );
}
