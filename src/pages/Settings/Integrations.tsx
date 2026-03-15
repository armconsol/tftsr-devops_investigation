import React from "react";
import { ExternalLink } from "lucide-react";
import { Card, CardHeader, CardTitle, CardContent, CardDescription, Badge } from "@/components/ui";

const integrations = [
  {
    name: "Confluence",
    description:
      "Automatically publish RCA and post-mortem documents to your Confluence workspace. Supports page creation, space selection, and template mapping.",
    features: ["Auto-publish documents", "Space & page selection", "Template mapping", "Version sync"],
  },
  {
    name: "ServiceNow",
    description:
      "Link triage sessions to ServiceNow incidents. Pull incident details for context, push resolution steps, and update incident status upon completion.",
    features: ["Incident linking", "Status sync", "Resolution push", "CMDB enrichment"],
  },
  {
    name: "Azure DevOps",
    description:
      "Create and link work items in Azure DevOps. Attach RCA documents to bug reports, create follow-up tasks from resolution steps, and sync status.",
    features: ["Work item creation", "Document attachment", "Task sync", "Pipeline triggers"],
  },
];

export default function Integrations() {
  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-3xl font-bold">Integrations</h1>
        <p className="text-muted-foreground mt-1">
          Connect TFTSR with your existing tools and platforms.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {integrations.map((integration) => (
          <Card key={integration.name} className="relative">
            <div className="absolute top-3 right-3">
              <Badge variant="secondary">Coming in v0.2</Badge>
            </div>
            <CardHeader>
              <CardTitle className="text-lg flex items-center gap-2">
                <ExternalLink className="w-4 h-4" />
                {integration.name}
              </CardTitle>
              <CardDescription>{integration.description}</CardDescription>
            </CardHeader>
            <CardContent>
              <ul className="space-y-1">
                {integration.features.map((feature) => (
                  <li
                    key={feature}
                    className="text-xs text-muted-foreground flex items-center gap-2"
                  >
                    <div className="w-1 h-1 rounded-full bg-muted-foreground" />
                    {feature}
                  </li>
                ))}
              </ul>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
