import clsx from 'clsx';
import { createElement } from 'react';
import { twMerge } from 'tailwind-merge';

// Components
import * as icons from '~/components/icons';
import { LogoIcon } from '~/components/icons/LogoIcon';
import { Section } from '~/components/Section';

// Data
import { categories } from '~/data/category';

const filteredCategories = categories.filter(category => category.value !== 'infrastructure');

function CommonCard({ className, children }: React.PropsWithChildren<{ className?: string; }>) {
  return (
    <div className={twMerge('flex flex-row text-lg font-medium text-zinc-800 bg-white items-center justify-center border border-zinc-200 rounded-xl h-18.5', className)}>
      {children}
    </div>
  );
}

function Separator({ className }: { className?: string; }) {
  return <div className={clsx('w-px h-full bg-zinc-300', className)} />;
}

export function HowItWorksSection() {
  return (
    <Section
      title="How it works"
      description="SuperNode acts as a unified control plane between your workloads and infrastructure, automating setup and deployment."
      className="bg-zinc-100"
    >
      <div className="grid grid-cols-[repeat(6,1fr)_auto_auto] items-center gap-8 w-full overflow-auto">
        {filteredCategories.map(category => (
          <div
            key={category.value}
            className="flex flex-col gap-1.5 text-sm font-medium text-zinc-800 bg-white items-center justify-center border border-zinc-200 rounded-md h-18.5 min-w-31"
          >
            {createElement(icons[category.icon], { className: 'size-6' })}
            <span className="text-nowrap">{category.label}</span>
          </div>
        ))}
        <Separator />
        <p className="text-2xl font-semibold text-zinc-800">Workloads</p>
        <CommonCard className="text-[#0000FF] border-[#0000FF] col-span-6 gap-4">
          <LogoIcon className="h-9.5" /> SuperNode Control Plane
        </CommonCard>
        <Separator className="row-span-2" />
        <p className="text-2xl font-semibold text-zinc-800 row-span-2">Orchestration</p>
        <CommonCard className="col-span-6">
          Kubernetes
        </CommonCard>
        <div className="grid grid-cols-4 col-span-6 gap-8">
          <CommonCard>Bare Metal</CommonCard>
          <CommonCard>AWS</CommonCard>
          <CommonCard>GCP</CommonCard>
          <CommonCard>Azure</CommonCard>
        </div>
        <Separator />
        <p className="text-2xl font-semibold text-zinc-800">Infrastructure</p>
      </div>
    </Section>
  );
}
