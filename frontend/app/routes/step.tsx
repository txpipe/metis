import { useState, type Dispatch, type SetStateAction } from 'react';
import { data, Outlet } from 'react-router';

// Components
import { SubHeader } from '~/components/SubHeader';
import { Sidebar, StepStatus } from '~/components/wizard/Sidebar';

// Context
import { WalletProvider } from '~/contexts/wallet';

import type { Route } from './+types/step';

export function loader({ request }: Route.LoaderArgs) {
  // Extrae el step del pathname si está disponible
  const pathname = new URL(request?.url || '').pathname;
  const stepMatch = pathname.match(/\/step\/(\d+)/);
  const step = stepMatch ? Number(stepMatch[1]) : null;

  if (!step || step < 1 || step > 5) {
    throw new Response('Page not found', { status: 404 });
  }

  return data({ step });
}

export type WizardStepContext = {
  setBadgeStatus: Dispatch<SetStateAction<string>>;
  setStepStatus: Dispatch<SetStateAction<StepStatus>>;
};

function initialStepStatus(step: number): StepStatus {
  // Step 3 will start with processing. This shouldn't occurs as the user will start always from step 1
  if (step === 3) return StepStatus.PROCESSING;
  return StepStatus.CURRENT;
}

export default function WizardStep({ loaderData }: Route.ComponentProps) {
  const { step } = loaderData;
  const [badgeStatus, setBadgeStatus] = useState(step === 1 ? 'Ready to start' : 'In-progress');
  const [stepStatus, setStepStatus] = useState<StepStatus>(initialStepStatus(step));

  return (
    <WalletProvider>
      <SubHeader
        logo="/images/midnight.svg"
        title="Midnight block producer node"
        subtitle="Become a Midnight Block Producer"
        tags={['Partner-chain']}
      />
      <div className="w-full border-b border-neutral-200 grid grid-cols-[350px_2px_1fr] flex-1">
        <Sidebar currentStep={{ number: step, status: stepStatus }} badgeStatus={badgeStatus} />
        <div className="w-full bg-[#F4F4F4]" />
        <section className="p-8">
          <Outlet context={{ setBadgeStatus, setStepStatus } satisfies WizardStepContext} />
        </section>
      </div>
    </WalletProvider>
  );
}
