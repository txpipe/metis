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
  slotNum: number | null;
  slotInEpoch: number | null;
  epochProgressPercent: number | null;
  epochTimeRemainingSeconds: number | null;
  tipRefSlot: number | null;
  tipDiffSlots: number | null;
  syncPercent: number | null;
  density: number | null;
  forks: number | null;
  txProcessed: number | null;
  pendingTx: number | null;
  pendingTxBytes: number | null;
  nodeVersion: string | null;
  nodeRevision: string | null;
  forgingEnabled: boolean | null;
  peersIncoming: number | null;
  peersOutgoing: number | null;
  connectionUniDir: number | null;
  connectionBiDir: number | null;
  connectionDuplex: number | null;
  inboundGovernorWarm: number | null;
  inboundGovernorHot: number | null;
  peerSelectionCold: number | null;
  peerSelectionWarm: number | null;
  peerSelectionHot: number | null;
  lastBlockDelaySeconds: number | null;
  blocksServed: number | null;
  blocksLate: number | null;
  blocksWithin1s: number | null;
  blocksWithin3s: number | null;
  blocksWithin5s: number | null;
  memLiveBytes: number | null;
  memHeapBytes: number | null;
  gcMinorCount: number | null;
  gcMajorCount: number | null;
  kesPeriod: number | null;
  kesRemaining: number | null;
  leaderCount: number | null;
  adoptedCount: number | null;
  forgedCount: number | null;
  aboutToLeadCount: number | null;
  invalidCount: number | null;
  missedSlots: number | null;
  kesExpirationSeconds: number | null;
  kesExpirationTime: string | null;
  errors: string[];
}

declare interface RegistryWorkload {
  repo: string;
  config: HelmChart['metadata'];
  namespace?: string;
  name?: string;
};
