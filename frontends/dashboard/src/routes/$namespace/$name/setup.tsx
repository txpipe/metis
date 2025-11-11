import { createFileRoute, Outlet, useLocation } from '@tanstack/react-router';

// Components
import { SubHeader } from '~/components/SubHeader';
import { Sidebar } from '~/components/wizard/Sidebar';

// Contexts
import { WalletProvider } from '~/contexts/wallet';
import { WizardProvider } from '~/contexts/wizard';

export const Route = createFileRoute('/$namespace/$name/setup')({
  // beforeLoad: async ({ params }) => {
  beforeLoad: async () => {
    // if (workloadId !== '2') {
    //   throw redirect({
    //     to: '/$workloadId',
    //     params: { workloadId },
    //   });
    // }
    return {};
  },
  component: SetupLayout,
});

function SetupLayout() {
  const location = useLocation();
  const stepMatch = location.pathname.match(/setup\/(\d+)/);
  const step = stepMatch ? Number(stepMatch[1]) : 1;

  return (
    <div className="grid grid-rows-[auto_1fr]">
      <WalletProvider>
        <WizardProvider step={step}>
          <SubHeader
            logo="/images/midnight.svg"
            title="Midnight block producer node"
            subtitle="Become a Midnight Block Producer"
            tags={['Partner-chain']}
          />
          <div className="w-full border-b border-neutral-200 grid grid-cols-[350px_2px_1fr]">
            <Sidebar step={step} />
            <div className="w-full bg-[#F4F4F4]" />
            <section className="p-8">
              <Outlet />
            </section>
          </div>
        </WizardProvider>
      </WalletProvider>
    </div>
  );
}
