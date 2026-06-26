import React, { Component, ReactNode } from "react";
import { AlertTriangle, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui";

interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: (error: Error, resetError: () => void) => ReactNode;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo): void {
    console.error("ErrorBoundary caught an error:", error, errorInfo);
  }

  resetError = (): void => {
    this.setState({ hasError: false, error: null });
  };

  render(): ReactNode {
    if (this.state.hasError && this.state.error) {
      if (this.props.fallback) {
        return this.props.fallback(this.state.error, this.resetError);
      }

      return (
        <div className="flex flex-col items-center justify-center h-full gap-6 p-8 text-center">
          <div className="flex items-center justify-center w-16 h-16 rounded-full bg-destructive/10">
            <AlertTriangle className="w-8 h-8 text-destructive" />
          </div>
          <div className="space-y-2">
            <h2 className="text-2xl font-semibold">Something went wrong</h2>
            <p className="text-muted-foreground max-w-md">
              An unexpected error occurred. You can try resetting the component or refreshing the page.
            </p>
          </div>
          <div className="space-y-4 w-full max-w-lg">
            <details className="text-left">
              <summary className="cursor-pointer text-sm font-medium text-muted-foreground hover:text-foreground">
                Error details
              </summary>
              <div className="mt-2 p-4 rounded-md bg-muted font-mono text-xs overflow-x-auto">
                <div className="font-semibold text-destructive mb-2">
                  {this.state.error.name}: {this.state.error.message}
                </div>
                {this.state.error.stack && (
                  <pre className="text-muted-foreground whitespace-pre-wrap break-all">
                    {this.state.error.stack}
                  </pre>
                )}
              </div>
            </details>
            <Button onClick={this.resetError} className="gap-2">
              <RefreshCw className="w-4 h-4" />
              Reset Component
            </Button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
