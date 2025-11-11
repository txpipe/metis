// Utils
import { runCommand } from '~/utils/process';

export async function install(namespace: string, name: string, image: string, version: string) {
  await runCommand(`
    helm install ${name} ${image} \
    --namespace ${namespace} \
    --version "${version}" \
    --set config.upstreamAddress=preview-node.world.dev.cardano.org:30002 \
    --set extraLabels.supernode/status=onboarding
  `.trim());
}
