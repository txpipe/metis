import { createFileRoute } from '@tanstack/react-router';
import { useState } from 'react';

// Components
import { CardWorkload } from '~/components/CardWorkload';
import { XIcon } from '~/components/icons/XIcon';
import { Button } from '~/components/ui/Button';
import { WorkloadsTable } from '~/components/WorkloadsTable';

export const Route = createFileRoute('/')({
  loader: async () => {
    const workloads = [
      {
        id: '12345',
        logoSrc: '/images/midnight.svg',
        name: 'Midnight Node',
        network: 'Mainnet',
        healthInfo: new Array(30).fill(1),
        uptime: 100,
        rewards: '300 Nights',
        status: 'connected' as Workload['status'],
      },
    ];

    return { workloads };
  },
  component: DashboardPage,
});

function DashboardPage() {
  const [showAvailableWorkloads, setShowAvailableWorkloads] = useState(false);
  const { workloads } = Route.useLoaderData();
  const [pendingWorkloads, setPendingWorkloads] = useState<typeof workloads[number][]>([]);

  const finalWorkloads = [...workloads, ...pendingWorkloads];

  return (
    <div className="mx-16 py-8">
      <h1 className="text-3xl/[40px] font-semibold text-[#2B2B2B]">Overview</h1>
      <div className="flex flex-row justify-between items-center mt-3">
        <p className="mt-3 text-[#42434D]">Manage and monitor all your workloads.</p>

        <Button
          type="button"
          onClick={() => {
            setShowAvailableWorkloads(true);
          }}
        >
          Add new workload
        </Button>
      </div>

      {showAvailableWorkloads && (
        <div className="mt-10 bg-[#F9F9F9] border-[0.5px] border-[#CBD5E1] rounded-xl p-6">
          <div className="flex flex-row justify-between items-center">
            <h2 className="text-[22px] font-semibold text-[#686868]">Add workloads</h2>
            <button type="button" onClick={() => setShowAvailableWorkloads(false)} className="cursor-pointer">
              <XIcon className="w-6.5 h-6.5 text-black" />
            </button>
          </div>
          <WorkloadsTable
            className="mt-8"
            onWorkloadSelected={workload => {
              setPendingWorkloads(prev => [
                ...prev,
                {
                  ...workload,
                  healthInfo: [],
                  uptime: 0,
                  rewards: '',
                  status: 'pending' as const,
                },
              ]);
            }}
          />
        </div>
      )}

      <div className="mt-10 bg-[#F9F9F9] border-[0.5px] border-[#CBD5E1] rounded-xl p-6">
        <h2 className="text-[22px] font-semibold text-[#686868]">My workloads</h2>

        <div className="grid grid-cols-1 lg:grid-cols-2 2xl:grid-cols-3 gap-7 mt-8">
          {finalWorkloads.map(workload => (
            <CardWorkload
              id={workload.id}
              key={workload.id}
              logoSrc={workload.logoSrc}
              name={workload.name}
              network={workload.network}
              status={workload.status}
              healthInfo={workload.healthInfo}
              uptime={workload.uptime}
              rewards={workload.rewards}
            />
          ))}
        </div>
      </div>
    </div>
  );
}
