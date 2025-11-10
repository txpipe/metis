import {
  AppsV1Api,
  CoreV1Api,
  KubeConfig,
  RbacAuthorizationV1Api,
  NetworkingV1Api,
  Watch,
  Log,
  Metrics,
  BatchV1Api,
  CustomObjectsApi,
} from '@kubernetes/client-node';

export function loadK8sConfig() {
  const kc = new KubeConfig();
  if (!!process.env.K8S_IN_CLUSTER) {
    kc.loadFromCluster();
  } else {
    kc.loadFromDefault();
  }
  return kc;
}

export function getClients() {
  const kc = loadK8sConfig();
  return {
    rbac: kc.makeApiClient(RbacAuthorizationV1Api),
    core: kc.makeApiClient(CoreV1Api),
    apps: kc.makeApiClient(AppsV1Api),
    net: kc.makeApiClient(NetworkingV1Api),
    batch: kc.makeApiClient(BatchV1Api),
    watch: new Watch(kc),
    log: new Log(kc),
    metrics: new Metrics(kc),
    crd: kc.makeApiClient(CustomObjectsApi),
    client: kc,
  };
}
