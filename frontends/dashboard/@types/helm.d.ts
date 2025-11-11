// Helm Release Types
interface HelmChart {
  metadata: {
    name: string;
    version: string;
    description?: string;
    appVersion?: string;
    apiVersion?: string;
    annotations?: Record<string, string>;
  };
  templates?: any[];
  values?: Record<string, any>;
}

interface HelmReleaseInfo {
  first_deployed: string;
  last_deployed: string;
  deleted?: string;
  description: string;
  status: 'unknown' | 'deployed' | 'uninstalled' | 'superseded' | 'failed' | 'uninstalling' | 'pending-install' | 'pending-upgrade' | 'pending-rollback';
  notes?: string;
}

interface HelmHook {
  name: string;
  kind: string;
  path: string;
  manifest: string;
  events: string[];
  last_run?: {
    started_at: string;
    completed_at: string;
    phase: string;
  };
}

interface DecodedHelmRelease {
  name: string;
  namespace: string;
  version: number;
  info: HelmReleaseInfo;
  chart: HelmChart;
  config?: Record<string, any>; // User-provided values
  manifest: string; // Generated Kubernetes manifests
  hooks?: HelmHook[];
}
