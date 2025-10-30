import { createFileRoute, useNavigate } from '@tanstack/react-router';
import { useEffect } from 'react';

// Components
import { Callout } from '~/components/ui/Callout';

// Wizard
import { StepStatus, useWizard } from '~/contexts/wizard';

export const Route = createFileRoute('/$workloadId/setup/3')({
  component: RouteComponent,
});

function RouteComponent() {
  const params = Route.useParams();
  const { setStepStatus } = useWizard();
  const navigate = useNavigate();

  useEffect(() => {
    setStepStatus(StepStatus.PROCESSING);

    // eslint-disable-next-line no-console
    console.log('Simulating wallet signature...');
    setTimeout(() => {
      navigate({ to: '/$workloadId/setup/4', params });
    }, 1000);

    return () => {
      setStepStatus(StepStatus.CURRENT);
    };
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <>
      <h2 className="text-[22px] text-[#42434D]">
        Registration is ready to submit on-chain.
      </h2>

      <Callout type="note" className="mt-10">
        <div>
          <span className="font-bold">Check your wallet</span> to sign and submit the registration.
        </div>
      </Callout>
    </>
  );
}
