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
  return <div className={clsx('w-px h-full bg-zinc-300 hidden min-[1210px]:block', className)} />;
}

export function HowItWorksSection() {
  return (
    <Section
      title="How it works"
      description="SuperNode acts as a unified control plane between your workloads and infrastructure, automating setup and deployment."
      className="bg-zinc-100"
    >
      <div className="grid grid-cols-3 lg:grid-cols-6 min-[1210px]:grid-cols-[repeat(6,1fr)_auto]! items-center gap-4 w-full overflow-auto">
        {filteredCategories.map(category => (
          <div
            key={category.value}
            className="flex flex-col gap-1.5 text-sm font-medium text-zinc-800 bg-white items-center justify-center border border-zinc-200 rounded-md h-18.5 min-w-31"
          >
            {createElement(icons[category.icon], { className: 'size-6' })}
            <span className="text-nowrap">{category.label}</span>
          </div>
        ))}
        <div className="hidden min-[1210px]:grid grid-cols-[auto_1fr] gap-6 self-stretch items-center pl-8">
          <Separator />
          <p className="text-2xl font-semibold text-zinc-800">Workloads</p>
        </div>
        <CommonCard className="text-[#FF007F] border-[#FF007F] col-span-3 lg:col-span-6 gap-3 text-lg font-medium">
          <div className="w-fit flex text-lg items-center gap-1.25 font-poppins text-zinc-900">
            <LogoIcon className="h-6.5" />
            <span>
              SUPER<span className="font-bold">NODE</span>
            </span>
          </div>
          <span>Control Plane</span>
        </CommonCard>
        <div className="row-span-2 hidden min-[1210px]:grid grid-cols-[auto_1fr] gap-6 self-stretch items-center pl-8">
          <Separator />
          <p className="text-2xl font-semibold text-zinc-800 hidden min-[1210px]:block">Orchestration</p>
        </div>
        <CommonCard className="col-span-3 lg:col-span-6">
          Kubernetes
        </CommonCard>
        <div className="grid grid-cols-2 lg:grid-cols-4 col-span-3 lg:col-span-6 gap-4">
          <CommonCard>Self-Hosted</CommonCard>
          <CommonCard>AWS</CommonCard>
          <CommonCard>GCP</CommonCard>
          <CommonCard>Azure</CommonCard>
        </div>
        <div className="hidden min-[1210px]:grid grid-cols-[auto_1fr] gap-6 self-stretch items-center pl-8">
          <Separator />
          <p className="text-2xl font-semibold text-zinc-800 hidden min-[1210px]:block">Infrastructure</p>
        </div>
      </div>
    </Section>
  );
}
