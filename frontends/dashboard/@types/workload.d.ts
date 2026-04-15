declare interface SupernodeAnnotations {
  displayName: string;
  icon: string;
  network: string;
  category?: string;
}

declare interface HelmWorkload {
  name: string;
  namespace: string;
  description?: string;
  chartName: string;
  chartVersion: string;
  status: HelmReleaseInfo['status'];
  supernodeStatus: 'onboarding' | 'ready';
  annotations?: SupernodeAnnotations;
  uptime?: UptimeEntry[];
  stsName?: string;
}

declare interface SimplifiedPod {
  name?: string;
  generateName?: string;
  namespace?: string;
  containerName?: string;
  chartName?: string;
  statusPhase?: string;
  hostname?: string;
  uptime: UptimeEntry[];
  metrics?: WorkloadMetrics;
  annotations?: SupernodeAnnotations;
}

declare type NodeRole = 'relay' | 'block-producer';

// Extensible union type for workload metrics.
declare type WorkloadMetrics = CardanoNodeMetrics;

declare interface CardanoNodeMetrics {
  type: 'cardano-node';
  role: NodeRole;
  blockHeight: number | null;
  epoch: number | null;
  slotInEpoch: number | null;
  density: number | null;
  txProcessed: number | null;
  pendingTx: number | null;
  pendingTxBytes: number | null;
  peersIncoming: number | null;
  peersOutgoing: number | null;
  lastBlockDelaySeconds: number | null;
  kesPeriod: number | null;
  kesRemaining: number | null;
  leaderCount: number | null;
  adoptedCount: number | null;
  invalidCount: number | null;
  missedSlots: number | null;
}

declare interface RegistryWorkload {
  repo: string;
  config: HelmChart['metadata'];
  namespace?: string;
  name?: string;
};
