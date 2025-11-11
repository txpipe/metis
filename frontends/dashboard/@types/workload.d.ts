declare interface HelmWorkload {
  name: string;
  namespace: string;
  description?: string;
  chartName: string;
  chartVersion: string;
  status: HelmReleaseInfo['status'];
  supernodeStatus: 'onboarding' | 'ready';
  nodeInfo?: {
    uid?: string;
    name?: string;
  };
  uptime?: UptimeEntry[];
}

declare interface SimplifiedPod {
  name?: string;
  generateName?: string;
  namespace?: string;
  containerName?: string;
  statusPhase?: string;
  hostname?: string;
  uptime: UptimeEntry[];
}

declare interface RegistryWorkload {
  repo: string;
  config: HelmChart['metadata'];
  namespace?: string;
  name?: string;
};
