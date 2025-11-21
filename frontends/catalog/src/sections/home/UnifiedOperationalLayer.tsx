import { useState } from 'react';

// Components
import { Section } from '~/components/Section';

const elements = [
  {
    label: 'Simplified provisioning',
    imgIcon: '/images/home/simplified-provisioning.svg',
    contentImg: '/images/home/simplified-provisioning-header.png',
    contentBody: 'SuperNode automates provisioning with a guided wizard that handles configuration, dependencies, and on-chain registration when required — so you can deploy any workload in minutes.',
  },
  {
    label: 'Version updates',
    imgIcon: '/images/home/version-updates.svg',
    contentImg: '/images/home/version-updates-header.png',
    contentBody: 'SuperNode keeps all your workloads up to date automatically, handling version upgrades and integrations so you can focus on operating—not maintaining.',
  },
  {
    label: 'Health monitoring',
    imgIcon: '/images/home/health-monitoring.svg',
    contentImg: '/images/home/health-monitoring-header.png',
    contentBody: 'SuperNode monitors the health and performance of every workload, giving you clear visibility and timely signals to keep operations running smoothly.',
  },
];

export function UnifiedOperationalLayerSection() {
  const [active, setActive] = useState(0);

  let activeItem = elements[active];

  return (
    <Section
      title="Unified operational layer"
      description="SuperNode brings all your workloads under one operational layer — provisioning, monitoring, updates, and more — handled automatically and in a consistent way."
    >
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="grid grid-rows-3 gap-8">
          {elements.map((item, index) => (
            <button
              type="button"
              key={item.label}
              className="relative border border-zinc-200 bg-white rounded-3xl p-8 flex items-center text-2xl/[1.32] font-semibold text-zinc-800 data-[active=true]:text-[#FF007F] data-[active=false]:opacity-40 cursor-pointer z-0"
              data-active={active === index}
              onClick={() => setActive(index)}
            >
              {item.label}
              <img src={item.imgIcon} alt={item.label} className="absolute right-4 bottom-0 -z-10" loading="lazy" />
            </button>
          ))}
        </div>

        <div className="border border-zinc-200 rounded-3xl p-8 flex flex-col justify-between">
          <img src={activeItem.contentImg} alt={activeItem.label} loading="lazy" />
          <p className="text-zinc-800">{activeItem.contentBody}</p>
        </div>
      </div>
    </Section>
  );
}
