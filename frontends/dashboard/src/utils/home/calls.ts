import { PassThrough } from 'stream';
import { createServerFn } from '@tanstack/react-start';
import { setResponseStatus } from '@tanstack/react-start/server';
import { z } from 'zod';

// Utils
import { getClients } from '~/utils/k8s';
import { getHelmReleases, getNetworkFromHelmRelease } from '~/utils/helm';
import { getStatefulSetUptime, emptyUptimeResult } from '~/utils/metrics';
import { getBlobDetails, searchWorkloads } from '~/utils/registry';
import { runCommand } from '~/utils/process';
import { nanoid } from '~/utils/generic';

// Installs
import { runInstall } from '~/utils/helm-install';

function getAnnotationsFromRelease(release: DecodedHelmRelease): SupernodeAnnotations | undefined {
  if (!release.chart.metadata.annotations) {
    return undefined;
  }

  return {
    displayName: release.chart.metadata.annotations['displayName'] || release.name,
    icon: release.chart.metadata.annotations['icon'],
    category: release.chart.metadata.annotations['category'],
    network: getNetworkFromHelmRelease(release),
  };
}

export const getServerWorkloads = createServerFn({
  method: 'GET',
}).handler(async (): Promise<HelmWorkload[]> => {
  const { core, apps } = getClients();

  const helmReleases = await getHelmReleases(core, 'all');

  let output: HelmWorkload[] = [];

  // Process helm releases
  for (const release of helmReleases) {
    const chartName = release.chart.metadata.name;
    const chartVersion = release.chart.metadata.version;

    let workload: HelmWorkload = {
      name: release.name,
      namespace: release.namespace,
      chartName,
      chartVersion,
      supernodeStatus: release.config?.extraLabels?.['supernode/status'] || 'ready',
      description: release.chart.metadata.description,
      status: release.info.status,
      annotations: getAnnotationsFromRelease(release),
    };

    if (workload.supernodeStatus === 'ready') {
      const listStatefulSet = await apps.listNamespacedStatefulSet({
        namespace: release.namespace,
        labelSelector: `helm.sh/chart=${chartName}-${chartVersion}`,
      });

      const nodeSts = listStatefulSet.items.find(sts => sts.spec?.serviceName?.endsWith('-headless'));
      if (!nodeSts) continue;

      workload.uptime = (!!nodeSts.metadata?.name)
        ? await getStatefulSetUptime(release.namespace, nodeSts.metadata.name)
        : emptyUptimeResult;

      workload.stsName = nodeSts.metadata?.name;
    }

    output.push(workload);
  }

  return output;
});

export const getServerWorkloadPods = createServerFn({
  method: 'GET',
}).inputValidator(
  (data: { namespace: string; name: string; }) => data,
).handler(async ({ data: { namespace, name } }) => {
  const { core, apps } = getClients();

  const sts = await apps.readNamespacedStatefulSet({
    name,
    namespace,
  }).catch(err => {
    // eslint-disable-next-line no-console
    console.error(err);
    return null;
  });

  if (!sts) {
    setResponseStatus(404);
    return { error: 'StatefulSet not found' };
  }

  const matchLabels = sts.spec?.selector.matchLabels;
  if (!matchLabels) {
    setResponseStatus(400);
    return { error: 'StatefulSet has no match labels' };
  }

  const stsChartVersion = sts.metadata?.labels?.['helm.sh/chart'];
  let helmRelease: DecodedHelmRelease | null = null;

  if (stsChartVersion) {
    const helmReleases = await getHelmReleases(core, namespace);
    helmRelease = helmReleases.find(hr => {
      const version = `${hr.chart.metadata.name}-${hr.chart.metadata.version}`;
      return version === stsChartVersion;
    }) ?? null;
  }

  const labelSelector = Object.entries(matchLabels)
    .map(([key, value]) => `${key}=${value}`)
    .join(',');

  const podsList = await core.listNamespacedPod({
    namespace,
    labelSelector,
  });

  const uptime = await getStatefulSetUptime(namespace, name);

  return {
    items: podsList.items.filter(p => !!p.metadata).map(pod => ({
      name: pod.metadata?.name,
      generateName: pod.metadata?.generateName,
      namespace: pod.metadata?.namespace,
      containerName: pod.spec?.containers?.[0].name,
      statusPhase: pod.status?.phase,
      hostname: pod.spec?.hostname,
      uptime,
      annotations: helmRelease ? getAnnotationsFromRelease(helmRelease) : undefined,
    } satisfies SimplifiedPod)),
  };
});

export const streamWorkloadPodLogs = createServerFn({
  method: 'GET',
}).inputValidator(
  (data: { namespace: string; podName: string; containerName: string; }) => data,
).handler(async ({ data: { namespace, podName, containerName } }): Promise<ReadableStream> => {
  const { log } = getClients();

  const nodeStream = new PassThrough();

  const textEncoder = new TextEncoder();

  const readableStream = new ReadableStream<Uint8Array>({
    start(controller) {
      nodeStream.on('data', chunk => {
        controller.enqueue(textEncoder.encode(chunk));
      });

      nodeStream.on('end', () => controller.close());
      nodeStream.on('error', err => controller.error(err));

      log.log(namespace, podName, containerName, nodeStream, {
        follow: true,
        tailLines: 100,
      }).catch(err => controller.error(err));
    },
    cancel() {
      nodeStream.destroy();
    },
  });

  return readableStream;
});

export const getAvailableWorkloads = createServerFn({
  method: 'GET',
}).handler(async (): Promise<RegistryWorkload[]> => {
  const workloadsImages = await searchWorkloads();

  // Zot Search GraphQL
  const zotRepos = workloadsImages.data?.GlobalSearch.Repos ?? [];

  const output: RegistryWorkload[] = [];

  for (const repo of zotRepos) {
    const firstConfig = repo?.NewestImage?.Manifests?.[0]?.ConfigDigest;
    if (firstConfig && repo.Name) {
      const config = await getBlobDetails<RegistryWorkload['config']>(repo.Name, firstConfig).catch();

      if (!config) continue;

      output.push({
        repo: repo.Name,
        config,
      });
    }
  }

  return output;
});

const AddWorkloadSchema = z.object({
  repo: z.string().min(1).startsWith('extensions/').regex(
    /^extensions\/[a-z0-9-]+$/,
    'Repo must contain only lowercase letters, numbers, and hyphens',
  ),
  name: z.string().min(1).max(63).regex(
    /^[a-z0-9]([-a-z0-9]*[a-z0-9])?$/,
    'Name must follow DNS-1123 subdomain format',
  ),
  version: z.string().min(1).regex(
    /^[0-9]+\.[0-9]+\.[0-9]+(-[a-z0-9]+(\.[a-z0-9]+)*)?$/,
    'Version must follow semantic versioning (e.g., 1.0.0 or 1.0.0-alpha.1)',
  ),
});

export const addWorkload = createServerFn({ method: 'POST' })
  .inputValidator(AddWorkloadSchema)
  .handler(async ({ data: { repo, name, version } }) => {
    if (!process.env.OCI_ENDPOINT) {
      setResponseStatus(500);
      return { error: 'Registry endpoint not configured' };
    }

    // oci://registry.example.com/scope/image
    const image = `${process.env.OCI_ENDPOINT}/${repo}`;

    const { core } = getClients();

    const namespace = `${name}-${nanoid(6)}`;

    // Create namespace
    await core.createNamespace({
      body: {
        metadata: {
          name: namespace,
        },
      },
    });

    await runInstall(repo, namespace, name, image, version);

    return { success: true, namespace, name };
  });

const DeleteWorkloadSchema = z.object({
  namespace: z.string().min(1).regex(
    /^[a-z0-9]([-a-z0-9]*[a-z0-9])?$/,
    'Namespace must follow DNS-1123 subdomain format',
  ),
  name: z.string().min(1).max(63).regex(
    /^[a-z0-9]([-a-z0-9]*[a-z0-9])?$/,
    'Name must follow DNS-1123 subdomain format',
  ),
});

export const deleteWorkload = createServerFn({ method: 'POST' })
  .inputValidator(DeleteWorkloadSchema)
  .handler(async ({ data: { namespace, name } }) => {
    await runCommand(`helm uninstall ${name} --namespace ${namespace}`.trim());

    const { core } = getClients();

    // Delete namespace
    await core.deleteNamespace({ name: namespace }).catch();

    return { success: true };
  });
