import AnsiToHtml from 'ansi-to-html';
import { type ReactNode, MouseEventHandler, useEffect, useRef, useState } from 'react';
import { queryOptions, useSuspenseQuery } from '@tanstack/react-query';
import { createFileRoute, Link, redirect } from '@tanstack/react-router';
import clsx from 'clsx';
import { twMerge } from 'tailwind-merge';
import toast from 'react-hot-toast';

// Components
import { CaretRightIcon } from '~/components/icons/CaretRightIcon';
import { GraphIcon } from '~/components/icons/GraphIcon';
import { Card, CardTitle } from '~/components/Card';
import { Button } from '~/components/ui/Button';
import { InfoTooltip } from '~/components/ui/InfoTooltip';
import { TrashIcon } from '~/components/icons/TrashIcon';
import { Toast } from '~/components/ui/Toast';

// Data
import { deleteWorkload, getServerWorkloadPods, streamWorkloadPodLogs } from '~/utils/home/calls';
import { getGrafanaDashboardUrl } from '~/utils/details/calls';
import {
  formatCountPair,
  formatDelaySeconds,
  formatEpochSlot,
  formatKesSummary,
  formatMetricValue,
  formatPeerCounts,
  formatPendingTx,
  formatRoleLabel,
} from '~/utils/metricsFormat';
import { calculateUptimePercentage } from '~/utils/metrics';
import { getStatusFromK8sStatus } from '~/utils/generic';

const textDecoder = new TextDecoder();

const metricDescriptions = {
  status: 'Current workload state reported by Kubernetes.',
  health: 'Percentage of the last 30 days this workload was healthy.',
  blockHeight: 'Latest block number observed by the node.',
  epochSlot: 'Current epoch and the slot within that epoch.',
  density: 'Recent chain density reported by the node, expressed as a percentage.',
  pendingTx: 'Transactions currently in the mempool, plus buffered size when available.',
  txProcessed: 'Total transactions processed by the node since startup.',
  peersInOut: 'Active inbound and outbound node connections.',
  lastBlockDelay: 'Latest observed block propagation delay.',
  blocksAdopted: 'Blocks successfully adopted by this producer since startup.',
  kesSummary: 'Current KES period and how many periods remain before rotation is required.',
  leaderAdopted: 'Leadership slots assigned to this producer versus blocks adopted.',
  invalidMissed: 'Forged but not adopted blocks, and scheduled slots the node missed.',
} as const;

async function getWorkloadDetails(namespace: string, name: string) {
  const data = await getServerWorkloadPods({ data: { namespace, name } });

  if (data.error || !data.items?.length) {
    throw redirect({
      to: '/',
    });
  }

  const dashboardUrl = await getGrafanaDashboardUrl({ data: { namespace } }).catch(err => {
    // eslint-disable-next-line no-console
    console.log(err);
    return null;
  });

  return {
    items: data.items,
    dashboardUrl,
  };
}

const workloadDetailsQueryOptions = (namespace: string, name: string) => queryOptions({
  queryKey: ['workloadDetails', namespace, name],
  queryFn: () => getWorkloadDetails(namespace, name),
  refetchInterval: 30000,
  refetchIntervalInBackground: true,
});

export const Route = createFileRoute('/$namespace/$name/')({
  loader: async ({ context, params }) => {
    await context.queryClient.ensureQueryData(workloadDetailsQueryOptions(params.namespace, params.name));
  },
  component: WorkloadIdInfo,
});

function InfoCard({
  label,
  value,
  valueClassName,
  description,
}: {
  label: string;
  value: string;
  valueClassName?: string;
  description?: string;
}) {
  return (
    <div className="flex flex-col items-center justify-center gap-3 py-4.5 px-6.5 rounded-xl border border-zinc-200 bg-white">
      <div className="flex flex-row gap-1 items-center text-[#969FAB] text-sm font-medium">
        {label}
        {description && <InfoTooltip content={description} />}
      </div>
      <div className={twMerge('text-sm text-[#2B2B2B]/80', valueClassName)}>{value}</div>
    </div>
  );
}

function MetricsSection({
  title,
  children,
  withDivider = false,
  aside,
}: {
  title: string;
  children: ReactNode;
  withDivider?: boolean;
  aside?: ReactNode;
}) {
  return (
    <div className={clsx(withDivider && 'border-t border-zinc-200 pt-6')}>
      <div className="mb-4 flex items-center justify-between gap-4">
        <div className="text-xs font-semibold uppercase tracking-[0.16em] text-[#64748B]">{title}</div>
        {aside}
      </div>
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-6">{children}</div>
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
  const { namespace, name } = Route.useParams();
  const workloadDetailsQuery = useSuspenseQuery(workloadDetailsQueryOptions(namespace, name));
  const { items, dashboardUrl } = workloadDetailsQuery.data;
  const logsContainerRef = useRef<HTMLDivElement>(null);
  const readableStreamRef = useRef<ReadableStreamDefaultReader<any> | null>(null);
  const activePod = items[0];
  const activePodNamespace = activePod?.namespace;
  const activePodContainerName = activePod?.containerName;
  const activePodKey = [activePod?.name, activePodNamespace, activePodContainerName].filter(Boolean).join('/');
  const [logState, setLogState] = useState(() => ({
    podKey: activePodKey,
    value: '',
  }));
  const logs = logState.podKey === activePodKey ? logState.value : '';

  useEffect(() => {
    if (logsContainerRef.current) {
      logsContainerRef.current.scroll({ top: logsContainerRef.current.scrollHeight, behavior: 'smooth' });
    }
  }, [logs]);

  useEffect(() => {
    const podName = activePod?.name;
    const podNamespace = activePodNamespace;
    const containerName = activePodContainerName;

    if (!podName || !podNamespace || !containerName) {
      return;
    }

    let cancelled = false;

    const streamLogs = async () => {
      const response = await streamWorkloadPodLogs({
        data: {
          podName,
          namespace: podNamespace,
          containerName,
        },
      });

      if (!response || cancelled) {
        return;
      }

      const reader = response.getReader();
      readableStreamRef.current = reader;

      try {
        let done = false;
        while (!done && !cancelled) {
          const { value, done: doneReading } = await reader.read();
          done = doneReading;
          if (!value) {
            continue;
          }

          const text = textDecoder.decode(value);
          if (!text) {
            continue;
          }

          setLogState(prev => ({
            podKey: activePodKey,
            value: ((prev.podKey === activePodKey ? prev.value : '') + text).slice(-10000),
          }));
        }
      } finally {
        if (readableStreamRef.current === reader) {
          readableStreamRef.current = null;
        }
      }
    };

    void streamLogs();

    return () => {
      cancelled = true;
      readableStreamRef.current?.cancel();
    };
  }, [activePod?.name, activePodContainerName, activePodKey, activePodNamespace]);

  if (!activePod) {
    return null;
  }

  const status = getStatusFromK8sStatus(activePod.statusPhase);
  const metrics = activePod.metrics;
  const cardanoNodeMetrics = metrics?.type === 'cardano-node' ? metrics : null;
  const isBlockProducer = cardanoNodeMetrics?.role === 'block-producer';

  return (
    <div className="mx-16 py-8 grid gap-10">
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
          {dashboardUrl && (
            <a
              href={dashboardUrl}
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
        <div className="space-y-6">
          <MetricsSection
            title="Node"
            aside={cardanoNodeMetrics && (
              <span className="rounded-full border border-zinc-200 bg-white px-3 py-1 text-xs font-medium text-[#64748B]">
                {formatRoleLabel(cardanoNodeMetrics.role)}
              </span>
            )}
          >
            <InfoCard
              label="Status"
              value={status}
              description={metricDescriptions.status}
              valueClassName={clsx('capitalize', {
                'text-[#69C876]': status === 'connected',
                'text-[#FF7474]': status === 'error',
                'text-[#2B2B2B]': status === 'paused',
                'text-[#0000FF]': status === 'pending',
              })}
            />
            <InfoCard
              label="Health"
              value={`${calculateUptimePercentage(activePod.uptime)}% uptime`}
              description={metricDescriptions.health}
            />
            {cardanoNodeMetrics && (
              <>
                <InfoCard
                  label="Block height"
                  value={formatMetricValue(cardanoNodeMetrics.blockHeight)}
                  description={metricDescriptions.blockHeight}
                />
                <InfoCard
                  label="Epoch / slot"
                  value={formatEpochSlot(cardanoNodeMetrics.epoch, cardanoNodeMetrics.slotInEpoch)}
                  description={metricDescriptions.epochSlot}
                />
                <InfoCard
                  label="Density"
                  value={cardanoNodeMetrics.density === null || cardanoNodeMetrics.density === undefined
                    ? '-'
                    : `${formatMetricValue(cardanoNodeMetrics.density, { maximumFractionDigits: 2 })}%`}
                  description={metricDescriptions.density}
                />
              </>
            )}
          </MetricsSection>

          {cardanoNodeMetrics && (
            <>
              <MetricsSection title="Mempool" withDivider>
                <InfoCard
                  label="Pending tx"
                  value={formatPendingTx(cardanoNodeMetrics.pendingTx, cardanoNodeMetrics.pendingTxBytes)}
                  description={metricDescriptions.pendingTx}
                />
                <InfoCard
                  label="Tx processed"
                  value={formatMetricValue(cardanoNodeMetrics.txProcessed)}
                  description={metricDescriptions.txProcessed}
                />
              </MetricsSection>

              <MetricsSection title="Network" withDivider>
                <InfoCard
                  label="Peers in / out"
                  value={formatPeerCounts(cardanoNodeMetrics.peersIncoming, cardanoNodeMetrics.peersOutgoing)}
                  description={metricDescriptions.peersInOut}
                />
                <InfoCard
                  label="Last block delay"
                  value={formatDelaySeconds(cardanoNodeMetrics.lastBlockDelaySeconds)}
                  description={metricDescriptions.lastBlockDelay}
                />
              </MetricsSection>

              {isBlockProducer && (
                <MetricsSection title="Producer" withDivider>
                  <InfoCard
                    label="Blocks adopted"
                    value={formatMetricValue(cardanoNodeMetrics.adoptedCount)}
                    description={metricDescriptions.blocksAdopted}
                  />
                  <InfoCard
                    label="KES current / remaining"
                    value={formatKesSummary(cardanoNodeMetrics.kesPeriod, cardanoNodeMetrics.kesRemaining)}
                    description={metricDescriptions.kesSummary}
                  />
                  <InfoCard
                    label="Leader / adopted"
                    value={formatCountPair(cardanoNodeMetrics.leaderCount, cardanoNodeMetrics.adoptedCount)}
                    description={metricDescriptions.leaderAdopted}
                  />
                  <InfoCard
                    label="Invalid / missed"
                    value={formatCountPair(cardanoNodeMetrics.invalidCount, cardanoNodeMetrics.missedSlots)}
                    description={metricDescriptions.invalidMissed}
                  />
                </MetricsSection>
              )}
            </>
          )}
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
