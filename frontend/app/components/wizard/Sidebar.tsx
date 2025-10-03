import clsx from 'clsx';

// Components
import { Badge } from '~/components/ui/Badge';
import { CheckIcon } from '~/components/icons/CheckIcon';
import { DotsIcon } from '~/components/icons/DotsIcon';

// Local
import { ConnectWallet } from './ConnectWallet';

export enum StepStatus {
  UPCOMING,
  CURRENT,
  PROCESSING,
  ERROR,
  COMPLETED,
}

interface Step {
  number: number;
  status: StepStatus;
}

interface Props {
  currentStep: Step;
  badgeStatus: string;
}

const steps = ['Connect wallet', 'Sign with your SPO key', 'Submit registration', 'Status'];

function getStepStatus(currentStep: Step, step: number): StepStatus {
  if (currentStep.number === step) {
    return currentStep.status;
  }

  if (step < currentStep.number) {
    return StepStatus.COMPLETED;
  }

  return StepStatus.UPCOMING;
}

function StepNumber({ number, stepStatus }: { number: number; stepStatus: StepStatus; }) {
  const hideStepNumber = stepStatus === StepStatus.COMPLETED || stepStatus === StepStatus.PROCESSING;
  return (
    <div
      className={clsx(
        'absolute left-0.75 flex items-center justify-center w-6.5 h-6.5 rounded-full font-medium text-sm',
        stepStatus === StepStatus.CURRENT && 'bg-[#2B2B2B] text-white',
        stepStatus === StepStatus.COMPLETED && 'bg-[#69C876] text-white',
        stepStatus === StepStatus.PROCESSING && 'bg-[#0000FF] text-white',
        stepStatus === StepStatus.UPCOMING && 'bg-white text-[#8D8D8D]',
      )}
    >
      {stepStatus === StepStatus.COMPLETED && <CheckIcon className="w-4.25 h-4.25" />}
      {stepStatus === StepStatus.PROCESSING && <DotsIcon className="w-4.25 h-4.25" />}
      {!hideStepNumber && <span>{number}</span>}
    </div>
  );
}

function getBadgeStyle(status: StepStatus) {
  if (status === StepStatus.ERROR) {
    return 'error';
  }

  if (status === StepStatus.COMPLETED) {
    return 'success';
  }

  return 'status';
}

export function Sidebar({ currentStep, badgeStatus }: Props) {
  return (
    <div className="flex flex-col bg-[#F9F9F9] py-8 pl-14 pr-8">
      <div className="text-[#2B2B2B] font-semibold text-xl">Onboarding Wizard</div>
      <p className="text-[#42434D] text-sm mt-3">Register as a candidate in the block producer committee.</p>
      <Badge
        label={badgeStatus}
        size="small"
        style={getBadgeStyle(currentStep.status)}
        className="mt-3"
      />
      <div className="flex-1 mt-8">
        <ul className="relative pl-12.5 space-y-8 before:absolute before:bg-[#D9D9D9]/44 before:rounded-full before:-top-0.75 before:-bottom-1.25 before:w-8 before:left-0">
          {steps.map((label, index) => {
            const stepStatus = getStepStatus(currentStep, index + 1);
            const isCurrent = stepStatus === StepStatus.CURRENT
              || stepStatus === StepStatus.PROCESSING
              || stepStatus === StepStatus.ERROR;

            return (
              <li key={label} className="group">
                <StepNumber number={index + 1} stepStatus={stepStatus} />
                <span
                  className={clsx(
                    'font-medium',
                    (isCurrent || stepStatus === StepStatus.COMPLETED) ? 'text-[#2B2B2B]' : 'text-[#8D8D8D]',
                  )}
                >
                  {label}
                </span>
              </li>
            );
          })}
        </ul>
      </div>

      <ConnectWallet variant="outlined" fullWidth />
    </div>
  );
}
