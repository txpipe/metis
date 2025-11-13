declare interface RegistryWorkload {
  repo: string;
  config: HelmChart['metadata'];
  namespace?: string;
  name?: string;
};
