import { createFileRoute, Outlet, useLocation } from '@tanstack/react-router';

// Components
import { SubHeader } from '~/components/SubHeader';
import { Sidebar } from '~/components/wizard/Sidebar';

// Context
import { WalletProvider } from '~/contexts/wallet';
import { WizardProvider } from '~/contexts/wizard';

export const Route = createFileRoute('/_steps')({
  component: StepsLayout,
});

function StepsLayout() {
  const location = useLocation();
  const stepMatch = location.pathname.match(/(\d+)/);
  const step = stepMatch ? Number(stepMatch[1]) : 1;

  return (
    <WalletProvider>
      <WizardProvider step={step}>
        <SubHeader
          logo="/images/midnight.svg"
          title="Midnight block producer node"
          subtitle="Become a Midnight Block Producer"
          tags={['Partner-chain']}
        />
        <div className="w-full border-b border-neutral-200 grid grid-cols-[350px_2px_1fr] flex-1">
          <Sidebar step={step} />
          <div className="w-full bg-[#F4F4F4]" />
          <section className="p-8">
            <Outlet />
          </section>
        </div>
      </WizardProvider>
    </WalletProvider>
  );
}
