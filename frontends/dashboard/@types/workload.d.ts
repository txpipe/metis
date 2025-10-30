declare interface Workload {
  id: string;
  logoSrc: string;
  name: string;
  network: string;
  description?: string;
  status: 'connected' | 'paused' | 'error' | 'pending';
  healthInfo?: number[];
  uptime?: number;
  rewards?: string;
}
