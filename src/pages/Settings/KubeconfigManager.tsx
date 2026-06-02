import { useState, useEffect } from 'react';
import { Upload, Check, Trash2, FileCode } from 'lucide-react';
import { Button, Card, CardHeader, CardTitle, CardContent, Badge } from '@/components/ui';
import {
  uploadKubeconfigCmd,
  listKubeconfigsCmd,
  activateKubeconfigCmd,
  deleteKubeconfigCmd,
  type KubeconfigInfo,
} from '@/lib/tauriCommands';

export default function KubeconfigManager() {
  const [configs, setConfigs] = useState<KubeconfigInfo[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [uploadContent, setUploadContent] = useState('');
  const [uploadName, setUploadName] = useState('');
  const [error, setError] = useState('');

  const loadConfigs = async () => {
    try {
      const data = await listKubeconfigsCmd();
      setConfigs(data);
    } catch (err) {
      setError(String(err));
    }
  };

  useEffect(() => {
    loadConfigs();
  }, []);

  const handleFileUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = async (event) => {
      const content = event.target?.result as string;
      setUploadContent(content);
      setUploadName(file.name.replace(/\.(yaml|yml)$/, ''));
    };
    reader.readAsText(file);
  };

  const handleUpload = async () => {
    if (!uploadContent || !uploadName) {
      setError('Please select a file and provide a name');
      return;
    }

    setIsLoading(true);
    setError('');
    try {
      await uploadKubeconfigCmd(uploadName, uploadContent);
      setUploadContent('');
      setUploadName('');
      await loadConfigs();
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  };

  const handleActivate = async (id: string) => {
    setIsLoading(true);
    setError('');
    try {
      await activateKubeconfigCmd(id);
      await loadConfigs();
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Are you sure you want to delete this kubeconfig?')) return;

    setIsLoading(true);
    setError('');
    try {
      await deleteKubeconfigCmd(id);
      await loadConfigs();
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-3xl font-bold mb-2">Kubeconfig Manager</h1>
        <p className="text-muted-foreground">
          Upload and manage multiple Kubernetes cluster configurations for kubectl commands
        </p>
      </div>

      {error && (
        <div className="rounded-lg border border-red-300 bg-red-50 p-4 text-sm text-red-800">
          {error}
        </div>
      )}

      {/* Upload Section */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Upload className="h-5 w-5" />
            Upload Kubeconfig
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <label className="block text-sm font-medium mb-2">Select File</label>
            <input
              type="file"
              accept=".yaml,.yml"
              onChange={handleFileUpload}
              className="block w-full text-sm text-muted-foreground file:mr-4 file:py-2 file:px-4 file:rounded file:border-0 file:text-sm file:font-semibold file:bg-primary file:text-primary-foreground hover:file:bg-primary/90"
            />
          </div>

          {uploadContent && (
            <>
              <div>
                <label htmlFor="config-name" className="block text-sm font-medium mb-2">
                  Configuration Name
                </label>
                <input
                  id="config-name"
                  type="text"
                  value={uploadName}
                  onChange={(e) => setUploadName(e.target.value)}
                  placeholder="e.g., production-cluster"
                  className="w-full px-3 py-2 border rounded-md"
                />
              </div>

              <div className="rounded-lg bg-slate-950 p-4 font-mono text-xs text-slate-400 max-h-60 overflow-y-auto">
                <pre>{uploadContent.substring(0, 500)}...</pre>
              </div>

              <Button onClick={handleUpload} disabled={isLoading} className="w-full">
                Upload Kubeconfig
              </Button>
            </>
          )}
        </CardContent>
      </Card>

      {/* Configs List */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <FileCode className="h-5 w-5" />
            Configured Clusters ({configs.length})
          </CardTitle>
        </CardHeader>
        <CardContent>
          {configs.length === 0 ? (
            <p className="text-sm text-muted-foreground text-center py-8">
              No kubeconfig files uploaded yet
            </p>
          ) : (
            <div className="space-y-3">
              {configs.map((config) => (
                <div
                  key={config.id}
                  className={`p-4 rounded-lg border ${
                    config.is_active
                      ? 'border-primary bg-primary/5'
                      : 'border-border'
                  }`}
                >
                  <div className="flex items-start justify-between">
                    <div className="space-y-1 flex-1">
                      <div className="flex items-center gap-2">
                        <h3 className="font-semibold">{config.name}</h3>
                        {config.is_active && (
                          <Badge variant="default" className="bg-green-600">
                            <Check className="h-3 w-3 mr-1" />
                            Active
                          </Badge>
                        )}
                      </div>
                      <div className="text-sm text-muted-foreground space-y-1">
                        <div>
                          <span className="font-medium">Context:</span> {config.context}
                        </div>
                        {config.cluster_url && (
                          <div>
                            <span className="font-medium">Cluster:</span> {config.cluster_url}
                          </div>
                        )}
                      </div>
                    </div>

                    <div className="flex gap-2">
                      {!config.is_active && (
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => handleActivate(config.id)}
                          disabled={isLoading}
                        >
                          Activate
                        </Button>
                      )}
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => handleDelete(config.id)}
                        disabled={isLoading}
                        className="text-red-600 hover:text-red-700 hover:bg-red-50"
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Info Card */}
      <Card>
        <CardHeader>
          <CardTitle>About Kubeconfig Files</CardTitle>
        </CardHeader>
        <CardContent className="space-y-2 text-sm text-muted-foreground">
          <p>
            Kubeconfig files contain authentication credentials and cluster connection details for
            kubectl commands.
          </p>
          <ul className="list-disc list-inside space-y-1 ml-2">
            <li>Upload your cluster's kubeconfig file (usually ~/.kube/config)</li>
            <li>Multiple clusters can be configured and switched between</li>
            <li>The active configuration is used for kubectl commands</li>
            <li>All kubeconfig files are encrypted using AES-256-GCM</li>
          </ul>
        </CardContent>
      </Card>
    </div>
  );
}
