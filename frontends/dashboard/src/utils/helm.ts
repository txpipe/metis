import { CoreV1Api } from '@kubernetes/client-node';
import pako from 'pako';

export const VALID_NODES = ['midnight'];

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

  const releases = [];

  for (const secret of secrets.items) {
    const decoded = decodeHelmSecret(secret.data || {});
    if (!decoded) continue;

    // Keep only nodes releases
    if (VALID_NODES.includes(decoded.chart.metadata.name)) {
      releases.push(decoded);
    }
  }

  return releases;
}
