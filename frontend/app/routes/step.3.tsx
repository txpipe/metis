import { useNavigate, useOutletContext } from 'react-router';
import { useEffect } from 'react';

// Components
import { Callout } from '~/components/ui/Callout';
import { StepStatus } from '~/components/wizard/Sidebar';

// Local
import type { WizardStepContext } from './step';

export default function WizardStep2() {
  const { setStepStatus } = useOutletContext<WizardStepContext>();
  const navigate = useNavigate();

  useEffect(() => {
    setStepStatus(StepStatus.PROCESSING);

    // eslint-disable-next-line no-console
    console.log('Simulating wallet signature...');
    setTimeout(() => {
      navigate('/step/4');
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
