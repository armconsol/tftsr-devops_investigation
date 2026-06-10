import React from "react";
import { Terminal } from "@/components/Kubernetes/Terminal";

export interface TerminalTabData {
  clusterId: string;
  namespace: string;
  podName?: string;
  containerName?: string;
}

interface TerminalTabProps {
  data: TerminalTabData;
}

/**
 * In-dock wrapper around the existing xterm-based Terminal component.
 * Delegates session management to Terminal itself.
 */
export function TerminalTab({ data }: TerminalTabProps) {
  return (
    <div className="h-full w-full p-2" data-testid="terminal-tab">
      <Terminal
        clusterId={data.clusterId}
        namespace={data.namespace}
        podName={data.podName}
        containerName={data.containerName}
      />
    </div>
  );
}
