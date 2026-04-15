import { PassThrough } from 'stream';
import {
  AppsV1Api,
  CoreV1Api,
  Exec,
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

export async function execPodCommand(
  namespace: string,
  podName: string,
  containerName: string,
  command: string | string[],
): Promise<{ stdout: string; stderr: string; }> {
  const kc = loadK8sConfig();
  const exec = new Exec(kc);
  const stdoutStream = new PassThrough();
  const stderrStream = new PassThrough();

  const stdoutChunks: Buffer[] = [];
  const stderrChunks: Buffer[] = [];

  stdoutStream.on('data', chunk => {
    stdoutChunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk));
  });

  stderrStream.on('data', chunk => {
    stderrChunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk));
  });

  let execStatus: { status?: string; message?: string; } | null = null;

  const ws = await exec.exec(
    namespace,
    podName,
    containerName,
    command,
    stdoutStream,
    stderrStream,
    null,
    false,
    status => {
      execStatus = status;
    },
  );

  await new Promise<void>((resolve, reject) => {
    const timeout = setTimeout(() => {
      reject(new Error(`Timed out waiting for exec command in pod ${namespace}/${podName}`));
    }, 15000);

    const cleanup = () => {
      clearTimeout(timeout);
      ws.off('close', handleClose);
      ws.off('error', handleError);
    };

    const handleClose = () => {
      cleanup();
      resolve();
    };

    const handleError = (error: Error) => {
      cleanup();
      reject(error);
    };

    ws.on('close', handleClose);
    ws.on('error', handleError);
  });

  const completedStatus = execStatus as { status?: string; message?: string; } | null;

  if (completedStatus && completedStatus.status === 'Failure') {
    throw new Error(completedStatus.message || `Exec command failed in pod ${namespace}/${podName}`);
  }

  return {
    stdout: Buffer.concat(stdoutChunks).toString('utf-8'),
    stderr: Buffer.concat(stderrChunks).toString('utf-8'),
  };
}
