declare namespace NodeJS {
  interface ProcessEnv {
    REGISTRY_ENDPOINT?: string;
    OCI_ENDPOINT?: string;
    BETA_SCRIPT_DEPLOYMENT_ID?: string;
    BETA_API_KEY?: string;
  }
}
