import { customAlphabet } from 'nanoid';

export type UIMappedStatus = 'connected' | 'paused' | 'error' | 'pending';

export function getStatusFromK8sStatus(
  k8sStatus: HelmReleaseInfo['status'] | SimplifiedPod['statusPhase'],
): UIMappedStatus {
  switch (k8sStatus) {
    // Active / healthy
    case 'deployed':
    case 'Running': // Pod Phase
      return 'connected';

    // Transitional / in progress
    case 'pending-install':
    case 'pending-upgrade':
    case 'pending-rollback':
    case 'uninstalling':
    case 'Pending': // Pod Phase
      return 'pending';

    // Not active but not an error (completed, superseded or uninstalled)
    case 'uninstalled':
    case 'Succeeded': // Pod Phase
    case 'superseded':
      return 'paused';

    // Error or unknown
    case 'failed':
    case 'Failed': // Pod Phase
    case 'Unknown': // Pod Phase
    case 'unknown':
      return 'error';

    default:
      return 'pending';
  }
}

export function getNodeDetails(nodeType: string) {
  switch (nodeType) {
    case 'midnight':
      return {
        displayName: 'Midnight Node',
        logoSrc: '/images/midnight.svg',
        logoAlt: 'Midnight Node Logo',
        rewards: '300 NIGHT',
        network: 'Preview',
      };
    default:
      return null;
  }
}

export type NodeDetails = ReturnType<typeof getNodeDetails>;

export const nanoid = customAlphabet('0123456789abcdefghijklmnopqrstuvwxyz', 6);
