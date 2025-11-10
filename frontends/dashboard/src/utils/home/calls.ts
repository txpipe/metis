import { PassThrough } from 'stream';
import { randomBytes } from 'crypto';
import { createServerFn } from '@tanstack/react-start';
import { setResponseStatus } from '@tanstack/react-start/server';
import { z } from 'zod';

// Utils
import { getClients } from '~/utils/k8s';
import { getHelmReleases } from '~/utils/helm';
import { getStatefulSetUptime, emptyUptimeResult } from '~/utils/metrics';
import { getBlobDetails, searchWorkloads } from '~/utils/registry';
import { runCommand } from '~/utils/process';
import { nanoid } from '~/utils/generic';

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
    };

    if (workload.supernodeStatus === 'ready') {
      const listStatefulSet = await apps.listNamespacedStatefulSet({
        namespace: release.namespace,
        labelSelector: `helm.sh/chart=${chartName}-${chartVersion}`,
      });

      const nodeSts = listStatefulSet.items.find(sts => sts.spec?.serviceName?.endsWith('-headless'));
      if (!nodeSts) continue;

      workload.nodeInfo = {
        uid: nodeSts.metadata?.uid,
        name: nodeSts.metadata?.name,
      };

      workload.uptime = (!!nodeSts.metadata?.name)
        ? await getStatefulSetUptime(release.namespace, nodeSts.metadata.name)
        : emptyUptimeResult;
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
  repo: z.string().min(1).startsWith('extensions/'),
  name: z.string().min(1),
  version: z.string().min(1),
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
          annotations: {
            'supernode.status': 'onboarding',
          },
        },
      },
    });

    const secret = randomBytes(32).toString('hex');
    const secretName = 'midnight-node-key';

    await core.createNamespacedSecret({
      namespace,
      body: {
        metadata: {
          name: secretName,
        },
        type: 'Opaque',
        stringData: {
          'node.key': secret,
        },
      },
    });

    // dbSync:
    //   managed:
    //     enabled: true
    //     nodeSocat:
    //       enabled: true
    //       targetHost: cardanonode1v3eujzxeqksxx8ukhh2.cardano-preview.cnode-m1.demeter.run
    //       targetPort: 9443

    await runCommand(`
      helm install ${name} ${image} \
      --namespace ${namespace} \
      --version "${version}" \
      --set nodeKey.existingSecret.name=${secretName} \
      --set nodeKey.existingSecret.key=node.key \
      --set persistence.size=5Gi \
      --set service.type=ClusterIP \
      --set extraLabels.supernode/status=onboarding
    `.trim());

    return { success: true, namespace, name };
  });

const DeleteWorkloadSchema = z.object({
  namespace: z.string().min(1),
  name: z.string().min(1),
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
