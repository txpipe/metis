import { runCommand, shellEscape } from '~/utils/process';

// Installs
import { install as midnightInstall } from '~/utils/helm-install/midnight';
import { install as dolosInstall } from '~/utils/helm-install/dolos';

export async function runInstall(
  repo: string,
  namespace: string,
  name: string,
  image: string,
  version: string,
) {
  if (repo.includes('midnight')) {
    return midnightInstall(namespace, name, image, version);
  }

  if (repo.includes('dolos')) {
    return dolosInstall(namespace, name, image, version);
  }

  return runCommand(`
    helm install ${shellEscape(name)} ${shellEscape(image)} \
    --namespace ${shellEscape(namespace)} \
    --version ${shellEscape(version)} \
    --set extraLabels.supernode/status=ready
  `.trim());
}
