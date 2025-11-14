import { createServerFn } from '@tanstack/react-start';
import { z } from 'zod';
// Utils
import { upgradeToReady } from '~/utils/helm';

const NamespaceArgs = z.object({
  namespace: z.string().min(1).regex(
    /^[a-z0-9]([-a-z0-9]*[a-z0-9])?$/,
    'Namespace must follow DNS-1123 subdomain format',
  ),
});

export const getGrafanaDashboardId = createServerFn({
  method: 'GET',
}).inputValidator(NamespaceArgs)
  .handler(async ({ data: { namespace } }) => {
    if (!process.env.GRAFANA_API_ENDPOINT) {
      throw new Error('GRAFANA_API_ENDPOINT is not defined');
    }

    // eslint-disable-next-line no-console
    console.log(`${process.env.GRAFANA_API_ENDPOINT}/search?query=${namespace}&type=dash-db`);
    const response = await fetch(`${process.env.GRAFANA_API_ENDPOINT}/search?query=${namespace}&type=dash-db`, {
      redirect: 'follow',

      headers: {
        'Content-Type': 'application/json',
        Authorization: 'Basic YWRtaW46YWRtaW4=',
      },
    });

    if (!response.ok) {
      throw new Error(`Failed to fetch Grafana dashboards: ${response.statusText}`);
    }

    const dashboards = await response.json();

    return dashboards.find(
      (dashboard: any) => dashboard.uid && dashboard.title.toLowerCase() === namespace.toLowerCase(),
    )?.uid || null;
  });

export const completeWorkflowSetup = createServerFn({
  method: 'POST',
}).inputValidator(NamespaceArgs)
  .handler(async ({ data: { namespace } }) => {
    await upgradeToReady(namespace);
    return { success: true };
  });
