type UtxoRef = { txHash: string; index: number; };

declare namespace NodeJS {
  interface ProcessEnv {
    PROMETHEUS_ENDPOINT?: string;
    REGISTRY_ENDPOINT?: string;
    OCI_ENDPOINT?: string;
  }
}
