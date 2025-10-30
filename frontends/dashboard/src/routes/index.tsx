import { createFileRoute } from '@tanstack/react-router';
import { useState } from 'react';

// Components
import { Card } from '~/components/Card';
import { CardWorkload } from '~/components/CardWorkload';
import { XIcon } from '~/components/icons/XIcon';
import { Button } from '~/components/ui/Button';
import { WorkloadsTable } from '~/components/WorkloadsTable';

export const Route = createFileRoute('/')({
  loader: async () => {
    const workloads: Workload[] = [
      {
        id: '12345',
        logoSrc: '/images/midnight.svg',
        name: 'Midnight Node',
        network: 'Mainnet',
        healthInfo: new Array(30).fill(1),
        uptime: 100,
        rewards: '300 Nights',
        status: 'connected',
      },
    ];

    return { workloads };
  },
  component: DashboardPage,
});

function DashboardPage() {
  const [showAvailableWorkloads, setShowAvailableWorkloads] = useState(false);
  const { workloads } = Route.useLoaderData();
  const [pendingWorkloads, setPendingWorkloads] = useState<Workload[]>([]);

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
        <Card
          title="Add workloads"
          titleAction={(
            <button type="button" onClick={() => setShowAvailableWorkloads(false)} className="cursor-pointer">
              <XIcon className="w-6.5 h-6.5 text-black" />
            </button>
          )}
          className="mt-10 gap-8"
        >
          <WorkloadsTable
            onWorkloadSelected={workload => {
              setPendingWorkloads(prev => [
                ...prev,
                {
                  ...workload,
                  healthInfo: [],
                  uptime: 0,
                  rewards: '',
                },
              ]);
            }}
          />
        </Card>
      )}

      <Card title="My workloads" className="mt-10 gap-8">
        <div className="grid grid-cols-1 lg:grid-cols-2 2xl:grid-cols-3 gap-7">
          {finalWorkloads.map(workload => (
            <CardWorkload
              workload={workload}
              key={workload.id}
            />
          ))}
        </div>
      </Card>
    </div>
  );
}
