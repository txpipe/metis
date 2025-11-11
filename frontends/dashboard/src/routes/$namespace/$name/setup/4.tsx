import { createFileRoute, Link } from '@tanstack/react-router';
import { useEffect, useState } from 'react';

// Components
import { CopyIcon } from '~/components/icons/CopyIcon';
import { Callout } from '~/components/ui/Callout';

// Context
import { StepStatus, useWizard } from '~/contexts/wizard';

export const Route = createFileRoute('/$namespace/$name/setup/4')({
  component: RouteComponent,
});

function RouteComponent() {
  const params = Route.useParams();
  const { setStepStatus, setBadgeStatus } = useWizard();
  const [success, setSuccess] = useState(false);

  useEffect(() => {
    setStepStatus(StepStatus.PROCESSING);

    // eslint-disable-next-line no-console
    console.log('Simulating success...');

    setTimeout(() => {
      setStepStatus(StepStatus.COMPLETED);
      setBadgeStatus('Completed');
      setSuccess(true);
    }, 1000);

    return () => {
      setStepStatus(StepStatus.CURRENT);
    };
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
  if (!success) {
    return (
      <>
        <h2 className="text-[22px] text-[#42434D]">
          Status on-chain.
        </h2>

        <Callout type="note" className="mt-10">
          <span className="font-bold">Processing registration.</span>
        </Callout>
      </>
    );
  }

  return (
    <>
      <h2 className="text-[22px] text-[#42434D] font-semibold">
        Registration successful!
      </h2>

      <div className="mt-10 bg-[#F9F9F9] border border-[#69C876] px-6 pt-3 pb-4.5 rounded-lg">
        <div className="flex justify-between items-center">
          <div className="text-[#69C876]">Tx Hash</div>
          <button
            type="button"
            className="flex gap-2 items-center text-sm h-10 cursor-pointer"
            onClick={() => {
              navigator.clipboard.writeText('4e9b1a5f8c0d74e2b6a92d3c11a7f4e0c8d35b26a54e9f19c3d0b78f25e6c123');
            }}
          >
            Copy <CopyIcon className="w-5 h-5" strokeWidth={1.5} />
          </button>
        </div>
        <p className="mt-2 text-[#565656]">
          4e9b1a5f8c0d74e2b6a92d3c11a7f4e0c8d35b26a54e9f19c3d0b78f25e6c123
        </p>
      </div>

      <p className="mt-10 text-[#2B2B2B]">
        At this point you are now registered as a candidate in the validator committee!<br />
        Access your <Link to="/$namespace/$name" params={params} className="text-[#0000FF] underline underline-offset-3 font-bold">Supernode Health Dashboard</Link> to check your Midnight block producer node's activity.
      </p>
    </>
  );
}
