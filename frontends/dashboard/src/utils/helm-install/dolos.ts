// Utils
import { runCommand } from '~/utils/process';

export async function install(namespace: string, name: string, image: string, version: string) {
  await runCommand(`
    helm install ${name} ${image} \
    --namespace ${namespace} \
    --version "${version}" \
    --set image.tag=v0.32.0 \
    --set extraLabels.supernode/status=ready \
    --set config.upstreamAddress=relay.cnode-m1.demeter.run:3002 \
    --set node.network=preview \
    --set node.networkMagic=2
  `.trim());
}
