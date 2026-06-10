import React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const badgeVariants = cva(
  "inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2",
  {
    variants: {
      variant: {
        default: "bg-primary text-primary-foreground hover:bg-primary/80",
        secondary: "bg-secondary text-secondary-foreground hover:bg-secondary/80",
        destructive: "bg-destructive text-destructive-foreground hover:bg-destructive/80",
        outline: "text-foreground border border-input hover:bg-accent",
        success: "bg-green-500 text-white hover:bg-green-600",
        warning: "bg-yellow-500 text-white hover:bg-yellow-600",
        info: "bg-blue-500 text-white hover:bg-blue-600",
        running: "bg-green-500 text-white",
        pending: "bg-yellow-500 text-white",
        failed: "bg-red-500 text-white",
        succeeded: "bg-blue-500 text-white",
        unknown: "bg-gray-500 text-white",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  }
);

export interface BadgeProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof badgeVariants> {
  icon?: React.ReactNode;
}

export function Badge({ className, variant, icon, children, ...props }: BadgeProps) {
  return (
    <div className={cn(badgeVariants({ variant }), className)} {...props}>
      {icon && <span className="mr-1 -ml-0.5">{icon}</span>}
      {children}
    </div>
  );
}

export function StatusBadge({
  status,
  className,
  ...props
}: Omit<BadgeProps, "variant"> & { status: string }) {
  const variant = getStatusVariant(status);
  return (
    <Badge variant={variant} className={className} {...props}>
      {status}
    </Badge>
  );
}

function getStatusVariant(status: string): BadgeProps["variant"] {
  const normalized = status.toLowerCase();
  if (normalized === "running" || normalized === "active" || normalized === "ready") {
    return "running";
  }
  if (normalized === "pending" || normalized === "waiting") {
    return "pending";
  }
  if (normalized === "failed" || normalized === "error") {
    return "failed";
  }
  if (normalized === "succeeded" || normalized === "completed" || normalized === "bound") {
    return "succeeded";
  }
  if (normalized.includes("crash") || normalized.includes("error") || normalized.includes("oom") || normalized.includes("backoff")) {
    return "failed";
  }
  if (normalized === "terminating" || normalized === "evicted") {
    return "destructive";
  }
  return "unknown";
}
