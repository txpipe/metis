import { randomBytes } from 'crypto';

// Utils
import { getClients } from '~/utils/k8s';
import { runCommand } from '~/utils/process';

export async function install(namespace: string, name: string, image: string, version: string) {
  const { core } = getClients();

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
}
