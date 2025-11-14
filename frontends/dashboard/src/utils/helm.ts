import { CoreV1Api } from '@kubernetes/client-node';
import pako from 'pako';
import merge from 'deepmerge';

// Utils
import { runCommand } from '~/utils/process';

function decodeHelmSecret(secretData: Record<string, string>): DecodedHelmRelease | null {
  const releaseData = secretData.release;
  if (!releaseData) return null;

  let releaseObj = null;

  try {
    // Double b64 and gzip decompression
    const firstDecode = Buffer.from(releaseData, 'base64');
    const secondDecode = Buffer.from(firstDecode.toString('utf-8'), 'base64');
    const decompressed = pako.inflate(secondDecode, { to: 'string' });
    releaseObj = JSON.parse(decompressed);
  } catch {
    try {
      // Single b64 and gzip decompression
      const decoded = Buffer.from(releaseData, 'base64');
      const decompressed = pako.inflate(decoded, { to: 'string' });
      releaseObj = JSON.parse(decompressed);
    } catch {
      try {
        // Single b64 decoding only
        const decoded = Buffer.from(releaseData, 'base64').toString('utf-8');
        releaseObj = JSON.parse(decoded);
      } catch {
        return null;
      }
    }
  }

  return releaseObj;
}

export async function getHelmReleases(api: CoreV1Api, namespace = 'all') {
  const secrets = await (namespace === 'all'
    ? api.listSecretForAllNamespaces({ fieldSelector: 'type=helm.sh/release.v1' })
    : api.listNamespacedSecret({ namespace, fieldSelector: 'type=helm.sh/release.v1' }));

  // Ensure that we have only the latest version of each release
  const latestSecretsMap = new Map<string, typeof secrets.items[0]>();

  for (const secret of secrets.items) {
    const secretNamespace = secret.metadata?.namespace;
    const secretName = secret.metadata?.labels?.name;
    const secretVersion = parseInt(secret.metadata?.labels?.version || '0', 10);

    if (!secretNamespace || !secretName) continue;

    const key = `${secretNamespace}:${secretName}`;
    const existing = latestSecretsMap.get(key);

    if (!existing) {
      latestSecretsMap.set(key, secret);
    } else {
      const existingVersion = parseInt(existing.metadata?.labels?.version || '0', 10);
      if (secretVersion > existingVersion) {
        latestSecretsMap.set(key, secret);
      }
    }
  }

  const releases = [];

  for (const secret of latestSecretsMap.values()) {
    const decoded = decodeHelmSecret(secret.data || {});
    if (!decoded) continue;

    // Keep only nodes releases
    if (decoded.chart.metadata.name != 'control-plane') {
      releases.push(decoded);
    }
  }

  return releases;
}

export function getNetworkFromHelmRelease(release: DecodedHelmRelease): string {
  const annotations = release.chart.metadata.annotations;
  if (!release.chart.values && !release.config) {
    return 'unknown';
  }

  const values = merge(
    release.chart.values || {},
    release.config || {},
    { arrayMerge: (_target, source) => source },
  );
  if (annotations && annotations.networkPath) {
    const parts = annotations.networkPath.split('.');
    let current: any = values;
    for (const part of parts) {
      if (current[part] !== undefined) {
        current = current[part];
      } else {
        break;
      }
    }
    if (typeof current === 'string') {
      return current;
    }
  }

  return 'unknown';
}

export async function upgradeToReady(namespace: string) {
  if (!process.env.OCI_ENDPOINT) {
    return;
  }

  const listCmd = `helm list --namespace ${namespace} --output json`;
  const releases = await runCommand(listCmd);
  const releasesJson = JSON.parse(releases);

  if (releasesJson.length === 0) {
    throw new Error(`No Helm releases found in namespace ${namespace}`);
  }

  const releaseName = releasesJson[0].name;

  const image = `${process.env.OCI_ENDPOINT}/extensions/${releaseName}`;

  if (releaseName && image) {
    await runCommand(`helm upgrade ${releaseName} ${image} --namespace ${namespace} --reuse-values --set extraLabels.supernode/status=ready`);
  }
}
