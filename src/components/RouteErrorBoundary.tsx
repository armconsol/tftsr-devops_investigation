import { ReactNode } from "react";
import { useLocation } from "react-router-dom";
import { AlertTriangle, RefreshCw } from "lucide-react";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { Button } from "@/components/ui";

interface RouteErrorBoundaryProps {
  children: ReactNode;
}

/**
 * Wraps routed page content in an {@link ErrorBoundary} that is keyed on the
 * current pathname. If a page throws during render, only the routed content is
 * replaced by a fallback — the surrounding application shell (navigation
 * sidebar, header, theme toggle) stays mounted so the user can navigate away
 * without restarting the app. Because the boundary is keyed on the pathname, it
 * is remounted (and therefore reset) automatically whenever the user navigates
 * to a different route.
 */
export function RouteErrorBoundary({ children }: RouteErrorBoundaryProps) {
  const location = useLocation();

  return (
    <ErrorBoundary
      key={location.pathname}
      fallback={(error, resetError) => (
        <div className="flex flex-col items-center justify-center gap-6 p-8 text-center">
          <div className="flex items-center justify-center w-16 h-16 rounded-full bg-destructive/10">
            <AlertTriangle className="w-8 h-8 text-destructive" />
          </div>
          <div className="space-y-2">
            <h2 className="text-2xl font-semibold">This page failed to load</h2>
            <p className="text-muted-foreground max-w-md">
              An unexpected error occurred while rendering this page. You can try
              again, or use the navigation menu to go somewhere else — the rest
              of the application is still available.
            </p>
          </div>
          <div className="space-y-4 w-full max-w-lg">
            <details className="text-left">
              <summary className="cursor-pointer text-sm font-medium text-muted-foreground hover:text-foreground">
                Error details
              </summary>
              <div className="mt-2 p-4 rounded-md bg-muted font-mono text-xs overflow-x-auto">
                <div className="font-semibold text-destructive mb-2">
                  {error.name}: {error.message}
                </div>
                {error.stack && (
                  <pre className="text-muted-foreground whitespace-pre-wrap break-all">
                    {error.stack}
                  </pre>
                )}
              </div>
            </details>
            <Button onClick={resetError} className="gap-2">
              <RefreshCw className="w-4 h-4" />
              Try Again
            </Button>
          </div>
        </div>
      )}
    >
      {children}
    </ErrorBoundary>
  );
}
