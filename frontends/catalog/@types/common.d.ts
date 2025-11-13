declare namespace NodeJS {
  interface ProcessEnv {
    REGISTRY_ENDPOINT?: string;
    VITE_OCI_ENDPOINT?: string;
  }
}

declare interface ImportMetaEnv {
  VITE_OCI_ENDPOINT?: string;
}
