import { useCallback, useEffect, useRef, useState } from 'react';
import { createFileRoute, Link, redirect } from '@tanstack/react-router';
import clsx from 'clsx';
import { twMerge } from 'tailwind-merge';

// Components
import { CaretRightIcon } from '~/components/icons/CaretRightIcon';
import { InfoCircleIcon } from '~/components/icons/InfoCircleIcon';
import { Card } from '~/components/Card';

// Data
import { getServerWorkloadPods, streamWorkloadPodLogs } from '~/utils/home/calls';
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

    return {
      items: data.items,
    };
  },
  component: WorkloadIdInfo,
});

function InfoChip({ label, value, valueClassName }: { label: string; value: string; valueClassName?: string; }) {
  return (
    <div className="flex flex-row items-center gap-2 py-1 px-3 rounded-full border-[0.5px] border-[#969FAB]">
      <div className="flex flex-row gap-1 items-center text-[#969FAB] text-sm font-medium">
        {label}
        <InfoCircleIcon className="w-3 h-3" />
      </div>
      <div className="w-px self-stretch bg-[#969FAB]" />
      <div className={twMerge('text-sm text-[#2B2B2B]/80', valueClassName)}>{value}</div>
    </div>
  );
}

function WorkloadIdInfo() {
  const { items } = Route.useLoaderData();
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
          setLogs(prev => prev + text);
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
    <div className="mx-16 py-8 grid grid-rows-[auto_auto_1fr] gap-5 max-h-[calc(100dvh-96px)]">
      <div className="flex items-center gap-2 text-[#64748B]">
        <Link to="/">Overview</Link>
        <CaretRightIcon className="w-4 h-4" />
        <span className="font-semibold text-[#2B2B2B]">{activePod.annotations?.displayName ?? activePod.containerName}</span>
      </div>

      <div className="flex flex-row items-start gap-4">
        <img src={activePod.annotations?.icon} alt={`${activePod.annotations?.displayName ?? activePod.containerName} logo`} className="w-15.5 h-15.5" />
        <div>
          <h1 className="text-[32px] font-semibold text-[#2B2B2B]">{activePod.annotations?.displayName ?? activePod.containerName}</h1>
          <span className="mt-1 text-[#969FAB] leading-none">{activePod.annotations?.network}</span>
          <div className="flex flex-row gap-2 mt-4 flex-wrap">
            <InfoChip
              label="Status"
              value={status}
              valueClassName={clsx('capitalize', {
                'text-[#69C876]': status === 'connected',
                'text-[#FF7474]': status === 'error',
                'text-[#2B2B2B]': status === 'paused',
                'text-[#0000FF]': status === 'pending',
              })}
            />
            <InfoChip label="Health" value={`${calculateUptimePercentage(activePod.uptime)}% uptime`} />
            <InfoChip label="Blocks produced" value="0" />
          </div>
        </div>
      </div>
      <div className="mt-5 max-h-full overflow-hidden min-h-[300px]">
        <Card title="Logs" className="gap-6 h-full">
          <div className="text-[13px] text-[#2B2B2B] bg-white rounded-xl p-6 min-h-0 h-full">
            <div className="overflow-y-auto whitespace-pre-wrap h-full font-mono" ref={logsContainerRef}>
              {logs || 'No logs available'}
            </div>
          </div>
        </Card>
      </div>
    </div>
  );
}
