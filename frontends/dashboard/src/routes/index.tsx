import { queryOptions, useSuspenseQuery } from '@tanstack/react-query';
import { createFileRoute } from '@tanstack/react-router';
import { useState } from 'react';
import { toast } from 'react-hot-toast';

// Components
import { Card } from '~/components/Card';
import { CardHelmWorkload } from '~/components/CardHelmWorkload';
import { XIcon } from '~/components/icons/XIcon';
import { Button } from '~/components/ui/Button';
import { Toast } from '~/components/ui/Toast';
import { WorkloadsTable } from '~/components/WorkloadsTable';

// Utils
import { getServerWorkloads } from '~/utils/home/calls';

const workloadsQueryOptions = () => queryOptions({
  queryKey: ['helmWorkloads'],
  queryFn: () => getServerWorkloads(),
});

export const Route = createFileRoute('/')({
  loader: async ({ context }) => {
    await context.queryClient.ensureQueryData(workloadsQueryOptions());
  },
  component: DashboardPage,
});

function DashboardPage() {
  const [showAvailableWorkloads, setShowAvailableWorkloads] = useState(false);
  const helmWorkloadsQuery = useSuspenseQuery(workloadsQueryOptions());

  const isEmpty = !helmWorkloadsQuery.data || helmWorkloadsQuery.data.length === 0;

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
            onWorkloadSelected={() => {
              setShowAvailableWorkloads(false);
              helmWorkloadsQuery.refetch();

              toast.custom(
                t => (
                  <Toast
                    toastId={t.id}
                    title="Workload Added"
                    message="Your workload was added successfully."
                    style="success"
                  />
                ),
              );
            }}
          />
        </Card>
      )}

      <Card title="My workloads" className="mt-10 gap-8">
        {isEmpty
          ? (
            <div className="grid grid-cols-1 min-h-[140px] items-center">
              <p className="text-[#42434D] text-xl text-center">
                Add your <button type="button" onClick={() => setShowAvailableWorkloads(true)} className="underline inline font-bold cursor-pointer">first workload</button>.
              </p>
            </div>
          )
          : (
            <div className="grid grid-cols-1 lg:grid-cols-2 2xl:grid-cols-3 gap-7">
              {helmWorkloadsQuery.data?.map(workload => (
                <CardHelmWorkload
                  key={workload.namespace}
                  workload={workload}
                  onDelete={() => helmWorkloadsQuery.refetch()}
                />
              ))}
            </div>
          )}
      </Card>
    </div>
  );
}
