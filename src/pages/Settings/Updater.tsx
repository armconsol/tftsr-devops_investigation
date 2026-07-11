import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/index';
import { Button } from '@/components/ui/index';
import { RefreshCw, Check, AlertCircle, Loader, ExternalLink } from 'lucide-react';
import {
  checkAppUpdatesCmd,
  installAppUpdatesCmd,
  type UpdateCheckResult,
} from '@/lib/tauriCommands';

export function Updater() {
  const [checking, setChecking] = useState(false);
  const [result, setResult] = useState<UpdateCheckResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const checkForUpdates = async () => {
    setChecking(true);
    setError(null);
    try {
      const data = await checkAppUpdatesCmd();
      setResult(data);
    } catch (err) {
      setError(String(err));
    } finally {
      setChecking(false);
    }
  };

  const handleDownloadUpdate = async () => {
    try {
      await installAppUpdatesCmd();
    } catch (err) {
      setError('Failed to open releases page: ' + String(err));
    }
  };

  useEffect(() => {
    void checkForUpdates();
  }, []);

  return (
    <div className="space-y-4">
      <div>
        <h1 className="text-2xl font-bold">Updater</h1>
        <p className="text-muted-foreground">Configure application updates</p>
      </div>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle>Check for Updates</CardTitle>
          <Button
            variant="outline"
            size="sm"
            onClick={checkForUpdates}
            disabled={checking}
          >
            {checking ? (
              <>
                <Loader className="mr-2 h-4 w-4 animate-spin" />
                Checking...
              </>
            ) : (
              <>
                <RefreshCw className="mr-2 h-4 w-4" />
                Check Now
              </>
            )}
          </Button>
        </CardHeader>
        <CardContent className="space-y-4">
          {error && (
            <div className="flex items-center space-x-2 rounded-lg bg-destructive/15 p-3 text-destructive">
              <AlertCircle className="h-4 w-4 flex-shrink-0" />
              <span className="text-sm">{error}</span>
            </div>
          )}

          {result && (
            <div className="text-sm text-muted-foreground space-y-1">
              <div>Current version: <span className="font-mono font-medium text-foreground">{result.currentVersion}</span></div>
              <div>Latest version: <span className="font-mono font-medium text-foreground">{result.latestVersion || '—'}</span></div>
            </div>
          )}

          {result?.updateAvailable ? (
            <div className="space-y-3">
              <div className="flex items-center justify-between rounded-lg bg-green-50 p-4 dark:bg-green-900/20">
                <div className="flex items-center space-x-3">
                  <div className="rounded-full bg-green-600 p-1 text-white">
                    <Check className="h-4 w-4" />
                  </div>
                  <div>
                    <div className="font-semibold text-green-900 dark:text-green-100">
                      Update Available — v{result.latestVersion}
                    </div>
                    <div className="text-sm text-green-700 dark:text-green-300">
                      Click below to open the releases page and download
                    </div>
                  </div>
                </div>
                <Button onClick={handleDownloadUpdate}>
                  <ExternalLink className="mr-2 h-4 w-4" />
                  Download Update
                </Button>
              </div>
              {result.releaseNotes && (
                <div className="rounded-lg border p-3 text-sm">
                  <div className="font-medium mb-1">Release Notes</div>
                  <pre className="whitespace-pre-wrap text-muted-foreground font-sans">{result.releaseNotes}</pre>
                </div>
              )}
            </div>
          ) : result ? (
            <div className="flex items-center space-x-3 rounded-lg bg-muted p-4">
              <div className="rounded-full bg-muted-foreground p-1 text-background">
                <Check className="h-4 w-4" />
              </div>
              <div>
                <div className="font-semibold">Up to Date</div>
                <div className="text-sm text-muted-foreground">
                  You are running the latest version
                </div>
              </div>
            </div>
          ) : null}
        </CardContent>
      </Card>
    </div>
  );
}
