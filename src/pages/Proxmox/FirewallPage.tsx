import React from 'react';
import { Button } from '@/components/ui/index';
import { RefreshCw } from 'lucide-react';
import { FirewallRuleList } from '@/components/Proxmox';

export function ProxmoxFirewallPage() {
  const rules = [
    { id: '1', rule: 100, action: 'ACCEPT', protocol: 'tcp', source: '192.168.1.0/24', destination: 'any', port: '22', status: 'enabled' },
    { id: '2', rule: 200, action: 'ACCEPT', protocol: 'tcp', source: 'any', destination: 'any', port: '80,443', status: 'enabled' },
    { id: '3', rule: 999, action: 'DROP', protocol: 'any', source: 'any', destination: 'any', status: 'enabled' },
  ];

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Firewall</h1>
          <p className="text-muted-foreground">Configure firewall rules</p>
        </div>
        <div className="flex space-x-2">
          <Button variant="outline" size="sm">
            <RefreshCw className="mr-2 h-4 w-4" />
            Refresh
          </Button>
        </div>
      </div>

      <FirewallRuleList
        rules={rules}
        onRefresh={() => {}}
      />
    </div>
  );
}
