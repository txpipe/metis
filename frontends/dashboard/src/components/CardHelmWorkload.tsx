import { MouseEventHandler, useState } from 'react';
import { Link } from '@tanstack/react-router';
import clsx from 'clsx';

// Components
import { InfoCircleIcon } from '~/components/icons/InfoCircleIcon';
import { AlertIcon } from '~/components/icons/AlertIcon';
import { Badge, Props as BadgeProps } from '~/components/ui/Badge';

// Utils
import { getStatusFromK8sStatus, UIMappedStatus } from '~/utils/generic';
import { calculateUptimePercentage } from '~/utils/metrics';
import { deleteWorkload } from '~/utils/home/calls';

interface Props {
  workload: HelmWorkload;
  onDelete?: () => void;
}

// Common badge props for different statuses
const badgePropsByStatus: Record<UIMappedStatus, BadgeProps> = {
  connected: { style: 'success', label: 'Connected' },
  paused: { style: 'pause', label: 'Paused' },
  error: { style: 'error', label: 'Error' },
  pending: { style: 'status', label: 'Onboarding' },
};

function WorkloadHeader({ workload }: Props) {
  const statusProps = workload.supernodeStatus === 'onboarding'
    ? badgePropsByStatus['pending']
    : badgePropsByStatus[getStatusFromK8sStatus(workload.status)];
  return (
    <div className="flex flex-row gap-3 items-center">
      {workload.annotations?.icon
        ? (
          <img src={workload.annotations.icon} alt={`${workload.annotations.displayName} Logo`} className="h-11 w-11" />
        )
        : (
          <div className="h-11 w-11 bg-[#E0E0E0] rounded-md" />
        )}
      <div className="flex-1">
        <h3 className="font-semibold text-lg text-[#2B2B2B] leading-none">{workload.annotations?.displayName ?? workload.name}</h3>
        <div className="text-[#969FAB] mt-1">{workload.annotations?.network}</div>
      </div>
      <Badge size="small" {...statusProps} />
    </div>
  );
}

function WorkloadReady({ workload }: Props) {
  return (
    <>
      {/* Health */}
      <div className="mt-8">
        <div className="flex font-medium items-center gap-1 text-[#969FAB]">
          Health <InfoCircleIcon className="w-3 h-3" />
        </div>
        <div className="mt-3">
          <div className="flex flex-row gap-1.5 justify-between h-7 w-full">
            {workload.uptime?.map(healthStatus => (
              <div
                key={`status-${healthStatus.date}`}
                className={clsx('w-full', {
                  'bg-[#69C876]': healthStatus.state === 1,
                  'bg-[#FF7474]': healthStatus.state === 0,
                  'bg-[#E0E0E0]': healthStatus.state === -1,
                })}
              />
            ))}
          </div>

          <div className="flex flex-row items-center gap-1 text-[#2B2B2B] text-sm mt-1">
            <span>30 days ago</span>
            <span className="flex-1 bg-[#969FAB] h-px" />
            <span>{calculateUptimePercentage(workload.uptime)}% uptime</span>
            <span className="flex-1 bg-[#969FAB] h-px" />
            <span>Today</span>
          </div>
        </div>
      </div>

      {/* Blocks produced */}
      <div className="mt-8">
        <div className="flex font-medium items-center gap-1 text-[#969FAB]">
          Blocks produced | Last 30 days <InfoCircleIcon className="w-3 h-3" />
        </div>
        <div className="mt-2 text-lg font-semibold text-[#2B2B2B]">
          0
        </div>
      </div>
    </>
  );
}

function WorkloadPending({ namespace, name, onDelete }: { namespace: string; name: string; onDelete?: Props['onDelete']; }) {
  const [deleting, setDeleting] = useState(false);

  const handleDelete: MouseEventHandler<HTMLButtonElement> = async e => {
    e.preventDefault();
    e.stopPropagation();

    setDeleting(true);
    try {
      await deleteWorkload({ data: { name, namespace } });
      onDelete?.();
    } catch (err) {
      // eslint-disable-next-line no-console
      console.error(err);
    } finally {
      setDeleting(false);
    }
  };

  return (
    <div className="flex-1 grid grid-rows-[1fr_auto] gap-6 mt-6">
      {/* Info box */}
      <div className="flex flex-row gap-3 items-center text-[#2B2B2B] bg-[#2B2B2B]/4 border-l-[3px] border-[#3F3F46] px-4 py-1.5 rounded-sm">
        <AlertIcon className="w-7 h-7" />
        <span className="text-sm leading-none">
          Complete your new workload setup to start earning rewards.
        </span>
      </div>

      {/* Actions */}
      <div className="flex flex-row justify-center items-center gap-2">
        <Link
          to="/$namespace/$name/setup"
          params={{ namespace, name }}
          className="text-[#0000FF] underline underline-offset-4"
        >
          Start onboarding wizard
        </Link>
        <span className="text-zinc-400">or</span>
        <button
          type="button"
          className="cursor-pointer disabled:cursor-not-allowed text-zinc-600"
          onClick={handleDelete}
          disabled={deleting}
        >
          {deleting ? 'Cancelling...' : 'Cancel'}
        </button>
      </div>
    </div>
  );
}

export function CardHelmWorkload({ workload, onDelete }: Props) {
  const className = 'bg-white rounded-3xl p-6 border border-zinc-200 col min-h-68.75';

  const name = workload.stsName ?? workload.name;

  if (workload.supernodeStatus === 'onboarding') {
    return (
      <div className={`${className} flex flex-col`}>
        <WorkloadHeader workload={workload} />

        <WorkloadPending namespace={workload.namespace} name={name} onDelete={onDelete} />
      </div>
    );
  }

  return (
    <Link
      to="/$namespace/$name"
      params={{ namespace: workload.namespace, name }}
      className={className}
    >
      <WorkloadHeader workload={workload} />

      <WorkloadReady workload={workload} />
    </Link>
  );
};
