import React from "react";
import { Check } from "lucide-react";

interface TriageProgressProps {
  currentLevel: number;
  totalLevels?: number;
}

export function TriageProgress({ currentLevel, totalLevels = 5 }: TriageProgressProps) {
  const steps = Array.from({ length: totalLevels }, (_, i) => i + 1);
  const showRootCause = currentLevel > totalLevels;

  return (
    <div className="px-4 py-3 border-b">
      <div className="flex items-center gap-1">
        {steps.map((step) => {
          const isCompleted = step < currentLevel;
          const isActive = step === currentLevel;

          return (
            <React.Fragment key={step}>
              <div className="flex flex-col items-center gap-1">
                <div
                  className={`w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold transition-colors ${
                    isCompleted
                      ? "bg-green-600 text-white"
                      : isActive
                        ? "bg-blue-600 text-white"
                        : "bg-muted text-muted-foreground"
                  }`}
                >
                  {isCompleted ? <Check className="w-4 h-4" /> : step}
                </div>
                <span className="text-[10px] text-muted-foreground font-medium">
                  Why {step}
                </span>
              </div>
              {step < totalLevels && (
                <div
                  className={`flex-1 h-0.5 mx-1 ${
                    step < currentLevel ? "bg-green-600" : "bg-muted"
                  }`}
                />
              )}
            </React.Fragment>
          );
        })}

        {/* Root cause indicator */}
        <div
          className={`flex-1 h-0.5 mx-1 ${
            showRootCause ? "bg-green-600" : "bg-muted"
          }`}
        />
        <div className="flex flex-col items-center gap-1">
          <div
            className={`w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold transition-colors ${
              showRootCause
                ? "bg-green-600 text-white"
                : "bg-muted text-muted-foreground"
            }`}
          >
            {showRootCause ? <Check className="w-4 h-4" /> : "RC"}
          </div>
          <span className="text-[10px] text-muted-foreground font-medium">
            Root Cause
          </span>
        </div>
      </div>
    </div>
  );
}
