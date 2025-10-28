declare interface Workload {
  id: string;
  logoSrc: string;
  name: string;
  network: string;
  status: 'connected' | 'paused' | 'error' | 'pending';
}
