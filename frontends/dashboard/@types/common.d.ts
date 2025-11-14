type UtxoRef = { txHash: string; index: number; };

declare namespace NodeJS {
  interface ProcessEnv {
    PROMETHEUS_ENDPOINT?: string;
    REGISTRY_ENDPOINT?: string;
    OCI_ENDPOINT?: string;
    GRAFANA_API_ENDPOINT?: string;
    VITE_GRAFANA_URL?: string;
  }
}

declare interface ImportMetaEnv {
  readonly VITE_GRAFANA_URL?: string;
}
