import { createServerFn } from '@tanstack/react-start';
import { createFileRoute, Link, redirect } from '@tanstack/react-router';
import clsx from 'clsx';
import { twMerge } from 'tailwind-merge';

// Components
import { useCallback, useEffect, useRef, useState } from 'react';
import { CaretRightIcon } from '~/components/icons/CaretRightIcon';
import { InfoCircleIcon } from '~/components/icons/InfoCircleIcon';
import { Card } from '~/components/Card';

// Data
import { workloads } from '~/data/workloads';

const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder();

const streamContainerLogs = createServerFn().handler(async () => {
  const { spawn } = await import('child_process');

  const containerId = '5f9dd23eed6d1dfce64d86515df04a25ea6054d02a0085d52fb7b971b46becfc';

  return new ReadableStream({
    start(controller) {
      const dockerProcess = spawn('docker', ['logs', '-f', '--tail', '100', containerId]);

      // Cuando hay datos en stdout
      dockerProcess.stdout.on('data', (data: Buffer) => {
        const text = data.toString();
        controller.enqueue(textEncoder.encode(text));
      });

      // Cuando hay datos en stderr (Docker también envía logs por stderr)
      dockerProcess.stderr.on('data', (data: Buffer) => {
        const text = data.toString();
        controller.enqueue(textEncoder.encode(text));
      });

      // Cuando hay un error
      dockerProcess.on('error', (error: Error) => {
        controller.enqueue(textEncoder.encode(error.message));
        controller.close();
      });

      // Cuando el proceso termina
      dockerProcess.on('close', () => {
        controller.close();
      });

      // Cleanup cuando se cancela el stream
      return () => {
        dockerProcess.kill();
      };
    },
    cancel() {
      // Esto se llama cuando el cliente cierra la conexión
    },
  });
});

export const Route = createFileRoute('/$workloadId/')({
  loader: async ({ params: { workloadId } }) => {
    const workload = workloads.find(w => w.id === workloadId);
    if (!workload) {
      throw redirect({
        to: '/',
      });
    }

    // Obtener logs del contenedor
    // const logs = await fetchContainerLogs();

    return {
      workload: {
        ...workload,
        rewards: '0 Nights',
        uptime: 100,
        status: 'connected',
      },
      // logs,
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
  // const { workload, logs: initialLogs } = Route.useLoaderData();
  const { workload } = Route.useLoaderData();
  const logsContainerRef = useRef<HTMLDivElement>(null);
  const readableStreamRef = useRef<ReadableStreamDefaultReader<any> | null>(null);
  const [logs, setLogs] = useState('');

  const startStreamLogs = useCallback(async () => {
    const response = await streamContainerLogs();

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
  }, []);

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
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <div className="mx-16 py-8 grid grid-rows-[auto_auto_1fr] gap-5 max-h-[calc(100dvh-96px)] overflow-y-scroll">
      <div className="flex items-center gap-2 text-[#64748B]">
        <Link to="/">Overview</Link>
        <CaretRightIcon className="w-4 h-4" />
        <span className="font-semibold text-[#2B2B2B]">{workload.name}</span>
      </div>

      <div className="flex flex-row items-start gap-4">
        <img src={workload.logoSrc} alt={`${workload.name} logo`} className="w-15.5 h-15.5" />
        <div>
          <h1 className="text-[32px] font-semibold text-[#2B2B2B]">{workload.name}</h1>
          <span className="mt-1 text-[#969FAB] leading-none">{workload.network}</span>
          <div className="flex flex-row gap-2 mt-4 flex-wrap">
            <InfoChip
              label="Status"
              value={workload.status}
              valueClassName={clsx('capitalize', {
                'text-[#69C876]': workload.status === 'connected',
                'text-[#FF7474]': workload.status === 'error',
                'text-[#2B2B2B]': workload.status === 'paused',
                'text-[#0000FF]': workload.status === 'pending',
              })}
            />
            <InfoChip label="Health" value={`${workload.uptime ?? 0}% uptime`} />
            <InfoChip label="Rewards" value={workload.rewards ?? ''} />
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
