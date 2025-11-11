import { MouseEventHandler, useState } from 'react';
import { Link } from '@tanstack/react-router';
import clsx from 'clsx';

// Components
import { InfoCircleIcon } from '~/components/icons/InfoCircleIcon';
import { Badge, Props as BadgeProps } from '~/components/ui/Badge';
import { AlertIcon } from '~/components/icons/AlertIcon';
import { ArrowRightIcon } from '~/components/icons/ArrowRightIcon';

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

function WorkloadHeader({ workload, onDelete }: Props) {
  const [deleting, setDeleting] = useState(false);
  const statusProps = workload.supernodeStatus === 'onboarding'
    ? badgePropsByStatus['pending']
    : badgePropsByStatus[getStatusFromK8sStatus(workload.status)];

  const handleDelete: MouseEventHandler<HTMLButtonElement> = async e => {
    e.preventDefault();
    e.stopPropagation();

    setDeleting(true);
    try {
      await deleteWorkload({ data: { name: workload.name, namespace: workload.namespace } });
      onDelete?.();
    } catch (err) {
      // eslint-disable-next-line no-console
      console.error(err);
    } finally {
      setDeleting(false);
    }
    // Implement delete functionality here
  };

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
      <button
        type="button"
        className="absolute bottom-4 right-4 rounded-full cursor-pointer disabled:cursor-not-allowed text-xs text-[#FF7474]"
        onClick={handleDelete}
        disabled={deleting}
      >
        {deleting ? 'Removing...' : 'Remove'}
      </button>
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

function WorkloadPending({ namespace, name }: { namespace: string; name: string; }) {
  return (
    <div className="flex-1 grid grid-rows-[1fr_auto] gap-6 mt-6">
      {/* Info box */}
      <div className="flex flex-row gap-3 items-center text-[#2B2B2B] bg-[#2B2B2B]/4 border-l-[3px] border-[#3F3F46] px-4 py-1.5 rounded-sm">
        <AlertIcon className="w-7 h-7" />
        <span className="text-sm leading-none">
          Complete your new workload setup to start earning rewards.
        </span>
      </div>

      <Link
        to="/$namespace/$name/setup"
        params={{ namespace, name }}
        className="flex gap-1.5 items-center w-fit mx-auto text-[#0000FF]"
      >
        <span className="underline underline-offset-4">
          Start onboarding wizard
        </span>
        <ArrowRightIcon className="w-4.5 h-2.5" />
      </Link>
    </div>
  );
}

export function CardHelmWorkload({ workload, onDelete }: Props) {
  const className = 'relative bg-white rounded-3xl p-6 shadow-[1px_0px_16px_0px_rgba(0,0,0,0.1)] flex flex-col min-h-68.75';

  if (workload.supernodeStatus === 'onboarding') {
    return (
      <div className={className}>
        <WorkloadHeader workload={workload} onDelete={onDelete} />

        <WorkloadPending namespace={workload.namespace} name={workload.name} />
      </div>
    );
  }

  return (
    <Link
      to="/$namespace/$name"
      params={{ namespace: workload.namespace, name: workload.name }}
      className={className}
    >
      <WorkloadHeader workload={workload} onDelete={onDelete} />

      <WorkloadReady workload={workload} />
    </Link>
  );
};
