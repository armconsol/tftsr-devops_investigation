import React from "react";
import type { HardwareInfo, ModelRecommendation } from "@/lib/tauriCommands";
import { Badge, Progress } from "@/components/ui";
import { Cpu, HardDrive, Monitor } from "lucide-react";

interface HardwareReportProps {
  hardware: HardwareInfo | null;
  recommendations: ModelRecommendation[];
}

export function HardwareReport({ hardware, recommendations }: HardwareReportProps) {
  if (!hardware) {
    return (
      <div className="text-sm text-muted-foreground p-4">
        Loading hardware information...
      </div>
    );
  }

  const maxRamDisplay = 64;
  const ramPercentage = Math.min(100, (hardware.total_ram_gb / maxRamDisplay) * 100);

  return (
    <div className="space-y-6">
      {/* Hardware Info */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {/* CPU */}
        <div className="flex items-start gap-3 rounded-lg border p-4">
          <Cpu className="w-5 h-5 text-muted-foreground mt-0.5" />
          <div>
            <p className="text-sm font-medium">CPU</p>
            <Badge variant="secondary" className="mt-1">
              {hardware.cpu_arch}
            </Badge>
          </div>
        </div>

        {/* RAM */}
        <div className="flex items-start gap-3 rounded-lg border p-4">
          <HardDrive className="w-5 h-5 text-muted-foreground mt-0.5" />
          <div className="flex-1">
            <p className="text-sm font-medium">RAM</p>
            <p className="text-xs text-muted-foreground mt-1 mb-2">
              {hardware.total_ram_gb.toFixed(1)} GB / {maxRamDisplay} GB
            </p>
            <Progress value={ramPercentage} />
          </div>
        </div>

        {/* GPU */}
        <div className="flex items-start gap-3 rounded-lg border p-4">
          <Monitor className="w-5 h-5 text-muted-foreground mt-0.5" />
          <div>
            <p className="text-sm font-medium">GPU</p>
            {hardware.gpu_vendor ? (
              <>
                <p className="text-xs mt-1">{hardware.gpu_vendor}</p>
                {hardware.gpu_vram_gb && (
                  <p className="text-xs text-muted-foreground">
                    {hardware.gpu_vram_gb} GB VRAM
                  </p>
                )}
              </>
            ) : (
              <p className="text-xs text-muted-foreground mt-1">No GPU detected</p>
            )}
          </div>
        </div>
      </div>

      {/* Model Recommendations */}
      {recommendations.length > 0 && (
        <div>
          <h4 className="text-sm font-medium mb-3">Model Recommendations</h4>
          <div className="space-y-2">
            {recommendations.map((rec) => (
              <div
                key={rec.name}
                className={`flex items-center justify-between rounded-lg border p-3 ${
                  rec.recommended ? "border-green-600 bg-green-50 dark:bg-green-950" : ""
                }`}
              >
                <div className="flex items-center gap-3">
                  {rec.recommended && (
                    <Badge className="bg-green-600 text-white">RECOMMENDED</Badge>
                  )}
                  <div>
                    <p className="text-sm font-medium">{rec.name}</p>
                    <p className="text-xs text-muted-foreground">
                      {rec.size} | Min RAM: {rec.min_ram_gb} GB
                    </p>
                  </div>
                </div>
                <p className="text-xs text-muted-foreground max-w-[200px] text-right">
                  {rec.description}
                </p>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
