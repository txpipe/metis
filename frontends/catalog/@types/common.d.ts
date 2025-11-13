declare namespace NodeJS {
  interface ProcessEnv {
    REGISTRY_ENDPOINT?: string;
    VITE_OCI_ENDPOINT?: string;
  }
}

declare interface ImportMetaE {
  VITE_OCI_ENDPOINT?: string;
}
