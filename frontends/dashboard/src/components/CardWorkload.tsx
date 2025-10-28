// Components
import { InfoCircleIcon } from '~/components/icons/InfoCircleIcon';
import { Badge, type Props as BadgeProps } from '~/components/ui/Badge';
import { AlertIcon } from './icons/AlertIcon';
import { ArrowRightIcon } from './icons/ArrowRightIcon';

interface ExistingWorkload extends Workload {
  status: 'connected' | 'paused' | 'error';
  healthInfo: number[];
  uptime: number;
  rewards: string;
}

interface PendingWorkload extends Workload {
  status: 'pending';
}

type Props = ExistingWorkload | PendingWorkload;

const badgePropsByStatus: Record<Workload['status'], BadgeProps> = {
  connected: { style: 'success', label: 'Connected' },
  paused: { style: 'pause', label: 'Paused' },
  error: { style: 'error', label: 'Error' },
  pending: { style: 'status', label: 'Onboarding' },
};

function BodyPending() {
  return (
    <div className="flex-1 grid grid-rows-[1fr_auto] gap-6 mt-6">
      {/* Info box */}
      <div className="flex flex-row gap-3 items-center text-[#2B2B2B] bg-[#2B2B2B]/4 border-l-[3px] border-[#3F3F46] px-4 py-1.5 rounded-sm">
        <AlertIcon className="w-7 h-7" />
        <span className="text-sm leading-none">
          Complete your new workload setup to start earning rewards.
        </span>
      </div>

      <div className="flex gap-1.5 items-center w-fit mx-auto text-[#0000FF]">
        <span className="underline underline-offset-4">
          Start onboarding wizard
        </span>
        <ArrowRightIcon className="w-4.5 h-2.5" />
      </div>
    </div>
  );
}

function BodyExisting({ healthInfo, uptime, rewards }: ExistingWorkload) {
  return (
    <>
      {/* Health */}
      <div className="mt-8">
        <div className="flex font-medium items-center gap-1 text-[#969FAB]">
          Health <InfoCircleIcon className="w-3 h-3" />
        </div>
        <div className="mt-3">
          <div className="flex flex-row gap-1.5 justify-between h-7 w-full">
            {healthInfo.map((healthStatus, index) => (
              <div key={`status-${index}`} className={`w-full ${healthStatus === 1 ? 'bg-[#69C876]' : 'bg-[#FF7474]'}`} />
            ))}
          </div>

          <div className="flex flex-row items-center gap-1 text-[#2B2B2B] text-sm mt-1">
            <span>30 days ago</span>
            <span className="flex-1 bg-[#969FAB] h-px" />
            <span>{uptime}% uptime</span>
            <span className="flex-1 bg-[#969FAB] h-px" />
            <span>Today</span>
          </div>
        </div>
      </div>

      {/* Rewards */}
      <div className="mt-8">
        <div className="flex font-medium items-center gap-1 text-[#969FAB]">
          Rewards | Last 30 days <InfoCircleIcon className="w-3 h-3" />
        </div>
        <div className="mt-2 text-lg font-semibold text-[#2B2B2B]">
          {rewards}
        </div>
      </div>
    </>
  );
}

export function CardWorkload({ logoSrc, name, network, status, ...rest }: Props) {
  return (
    <div className="bg-white rounded-3xl p-6 shadow-[1px_0px_16px_0px_rgba(0,0,0,0.1)] flex flex-col min-h-68.75">
      {/* Workload Info */}
      <div className="flex flex-row gap-3 items-center">
        <img src={logoSrc} alt={`${name} Logo`} className="h-11 w-11" />
        <div className="flex-1">
          <h3 className="font-semibold text-lg text-[#2B2B2B] leading-none">{name}</h3>
          <div className="text-[#969FAB] mt-1">{network}</div>
        </div>
        <Badge size="small" {...badgePropsByStatus[status]} />
      </div>

      {status === 'pending'
        ? <BodyPending />
        : <BodyExisting {...rest as ExistingWorkload} />}
    </div>
  );
};
