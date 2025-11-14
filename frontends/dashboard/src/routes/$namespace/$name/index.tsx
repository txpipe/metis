import AnsiToHtml from 'ansi-to-html';
import { MouseEventHandler, useCallback, useEffect, useRef, useState } from 'react';
import { createFileRoute, Link, redirect } from '@tanstack/react-router';
import clsx from 'clsx';
import { twMerge } from 'tailwind-merge';

// Components
import toast from 'react-hot-toast';
import { CaretRightIcon } from '~/components/icons/CaretRightIcon';
import { InfoCircleIcon } from '~/components/icons/InfoCircleIcon';
import { GraphIcon } from '~/components/icons/GraphIcon';
import { Card, CardTitle } from '~/components/Card';
import { Button } from '~/components/ui/Button';
import { TrashIcon } from '~/components/icons/TrashIcon';
import { Toast } from '~/components/ui/Toast';

// Data
import { deleteWorkload, getServerWorkloadPods, streamWorkloadPodLogs } from '~/utils/home/calls';
import { getGrafanaDashboardId } from '~/utils/details/calls';
import { calculateUptimePercentage } from '~/utils/metrics';
import { getStatusFromK8sStatus } from '~/utils/generic';

const textDecoder = new TextDecoder();

export const Route = createFileRoute('/$namespace/$name/')({
  loader: async ({ params }) => {
    const data = await getServerWorkloadPods({ data: params });

    if (data.error || !data.items?.length) {
      throw redirect({
        to: '/',
      });
    }
    const dashboardId = !!import.meta.env.VITE_GRAFANA_URL
      ? await getGrafanaDashboardId({ data: { namespace: params.namespace } }).catch(() => null)
      : null;

    return {
      items: data.items,
      dashboardId,
    };
  },
  component: WorkloadIdInfo,
});

function InfoCard({ label, value, valueClassName }: { label: string; value: string; valueClassName?: string; }) {
  return (
    <div className="flex flex-col items-center justify-center gap-3 py-4.5 px-6.5 rounded-xl border border-zinc-200 bg-white">
      <div className="flex flex-row gap-1 items-center text-[#969FAB] text-sm font-medium">
        {label}
        <InfoCircleIcon className="w-3 h-3" />
      </div>
      <div className={twMerge('text-sm text-[#2B2B2B]/80', valueClassName)}>{value}</div>
    </div>
  );
}

const converter = new AnsiToHtml({
  newline: false,
  escapeXML: true,
  stream: true,
});

function DeleteAction() {
  const { namespace, name } = Route.useParams();
  const navigate = Route.useNavigate();

  const [deleting, setDeleting] = useState(false);

  const handleDelete: MouseEventHandler<HTMLButtonElement> = async e => {
    e.preventDefault();
    e.stopPropagation();

    setDeleting(true);
    try {
      await deleteWorkload({ data: { name, namespace } });
      navigate({ to: '/' });
      toast.custom(
        t => (
          <Toast
            toastId={t.id}
            title="Workload Deleted"
            message="Your workload was deleted successfully."
            style="success"
          />
        ),
      );
    } catch (err) {
      // eslint-disable-next-line no-console
      console.error(err);
    } finally {
      setDeleting(false);
    }
  };

  return (
    <Button
      type="button"
      variant="outlined"
      color="red"
      text="sm"
      className="gap-2 leading-none px-6"
      disabled={deleting}
      onClick={handleDelete}
    >
      <TrashIcon className="size-4" />
      {deleting ? <span>Deleting...</span> : <span>Delete workload</span>}
    </Button>
  );
}

function WorkloadIdInfo() {
  const { namespace } = Route.useParams();
  const { items, dashboardId } = Route.useLoaderData();
  const logsContainerRef = useRef<HTMLDivElement>(null);
  const readableStreamRef = useRef<ReadableStreamDefaultReader<any> | null>(null);
  const [logs, setLogs] = useState('');
  const [activePod, _setActivePod] = useState<SimplifiedPod>(items[0]);

  const startStreamLogs = useCallback(async () => {
    if (!activePod || (!activePod.name || !activePod.namespace || !activePod.containerName)) {
      return;
    }

    const response = await streamWorkloadPodLogs({
      data: {
        podName: activePod.name || '',
        namespace: activePod.namespace || '',
        containerName: activePod.containerName || '',
      },
    });

    if (!response) {
      return;
    }

    readableStreamRef.current = response.getReader();
    let done = false;
    while (!done) {
      const { value, done: doneReading } = await readableStreamRef.current.read();
      done = doneReading;
      if (value) {
        const text = textDecoder.decode(value);
        if (text) {
          setLogs(prev => (prev + text).slice(-10000)); // Keep only last 10k characters
        }
      }
    }
  }, [activePod]);

  useEffect(() => {
    if (logsContainerRef.current) {
      logsContainerRef.current.scroll({ top: logsContainerRef.current.scrollHeight, behavior: 'smooth' });
    }
  }, [logs]);

  useEffect(() => {
    startStreamLogs();
    return () => {
      readableStreamRef.current?.cancel();
    };
  }, [startStreamLogs]);

  const status = getStatusFromK8sStatus(activePod.statusPhase);

  return (
    <div className="mx-16 py-8 grid grid-rows-[auto_auto_auto_1fr_auto] gap-10">
      <div>
        <div className="flex items-center gap-2 text-[#64748B]">
          <Link to="/">Overview</Link>
          <CaretRightIcon className="w-4 h-4" />
          <span className="font-semibold text-[#2B2B2B]">{activePod.annotations?.displayName ?? activePod.containerName}</span>
        </div>

        <div className="flex flex-row items-start gap-4 mt-4">
          <img src={activePod.annotations?.icon} alt={`${activePod.annotations?.displayName ?? activePod.containerName} logo`} className="w-15.5 h-15.5" />
          <div className="grow">
            <h1 className="text-[32px] font-semibold text-[#2B2B2B]">{activePod.annotations?.displayName ?? activePod.containerName}</h1>
            <span className="mt-1 text-[#969FAB] leading-none">{activePod.annotations?.network}</span>
          </div>
          {dashboardId && (
            <a
              href={`${import.meta.env.VITE_GRAFANA_URL}/d/${dashboardId}/${namespace}`}
              target="_blank"
              rel="noopener noreferrer"
              className="border border-zinc-900 rounded-full py-2.5 px-6 text-sm/none flex items-center justify-center gap-1 self-end"
            >
              <GraphIcon className="size-4" strokeWidth={2} />
              <span>Open Grafana</span>
            </a>
          )}
        </div>
      </div>
      <Card>
        <div className="grid grid-flow-col auto-cols-fr w-max gap-6">
          <InfoCard
            label="Status"
            value={status}
            valueClassName={clsx('capitalize', {
              'text-[#69C876]': status === 'connected',
              'text-[#FF7474]': status === 'error',
              'text-[#2B2B2B]': status === 'paused',
              'text-[#0000FF]': status === 'pending',
            })}
          />
          <InfoCard label="Health" value={`${calculateUptimePercentage(activePod.uptime)}% uptime`} />
          <InfoCard label="Blocks produced" value="0" />
        </div>
      </Card>

      <div className="overflow-hidden h-138">
        <Card className="gap-6 h-full">
          <CardTitle>Logs</CardTitle>
          <div className="text-[13px] text-[#2B2B2B] bg-white rounded-xl p-6 min-h-0 h-full">
            <div
              className="overflow-y-auto whitespace-pre-wrap h-full font-mono"
              ref={logsContainerRef}
              dangerouslySetInnerHTML={{ __html: converter.toHtml(logs || 'No logs available').trim() }}
            />
          </div>
        </Card>
      </div>

      <Card className="flex flex-row items-center justify-between">
        <div>
          <CardTitle>Delete workload</CardTitle>
          <p className="mt-3 text-[13px]/none text-[#2B2B2B]">Once you delete a workload, there is no going back. Please be certain.</p>
        </div>

        <DeleteAction />
      </Card>
    </div>
  );
}
