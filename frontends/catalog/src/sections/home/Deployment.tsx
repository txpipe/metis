import clsx from 'clsx';
import { Server2Icon } from '~/components/icons/Server2Icon';
import { Section } from '~/components/Section';
import { button } from '~/components/ui/Button';

function CloudProviderCard({ children, className }: React.PropsWithChildren<{ className?: string; }>) {
  return (
    <div className={clsx('flex flex-col items-center justify-center gap-4 border border-zinc-200 px-6 py-7', className)}>
      {children}
    </div>
  );
}

export function DeploymentSection() {
  return (
    <Section
      title="Choose your own deployment"
      description="Supernode can run on bare metal or any cloud provider. We take care of provisioning and environment setup, you just choose where to run your workloads."
    >
      <div className="grid auto-cols-fr grid-flow-col gap-4">
        <CloudProviderCard className="rounded-l-4xl">
          <Server2Icon strokeWidth={1} className="size-10.5 text-zinc-900" />
          <p className="text-center text-[44px]/[1.1] font-semibold">
            Bare<br />metal
          </p>
        </CloudProviderCard>
        <CloudProviderCard>
          <img src="/images/cloud-provider/aws-logo.png" alt="AWS Logo" className="w-full max-w-[142px]" />
        </CloudProviderCard>
        <CloudProviderCard>
          <img src="/images/cloud-provider/google-cloud-logo.png" alt="Google Cloud Logo" className="w-full max-w-[196px]" />
        </CloudProviderCard>
        <CloudProviderCard className="rounded-r-4xl">
          <img src="/images/cloud-provider/azure-logo.png" alt="Azure Logo" className="w-full max-w-[125px]" />
        </CloudProviderCard>
      </div>
      <div className="flex items-center justify-center gap-6 text-lg text-[#0E3550]">
        <p>
          Ready to run your own <span className="font-bold">Supernode</span>?
        </p>
        <a
          href="#beta"
          className={button({ variant: 'outlined', fullWidth: true, className: 'max-w-[210px]' })}
        >
          Deploy
        </a>
      </div>
    </Section>
  );
}
