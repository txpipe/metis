import AnsiToHtml from 'ansi-to-html';
import {
  type ReactNode,
  MouseEventHandler,
  useEffect,
  useRef,
  useState,
} from 'react';
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
import {
  deleteWorkload,
  getServerWorkloadPods,
  streamWorkloadPodLogs,
} from '~/utils/home/calls';
import { getGrafanaDashboardUrl } from '~/utils/details/calls';
import {
  formatBooleanMetric,
  formatBytesToGiB,
  formatCountPair,
  formatCountTriplet,
  formatDelaySeconds,
  formatDurationSeconds,
  formatKesSummary,
  formatMetricValue,
  formatPeerCounts,
  formatPendingTx,
  formatPercent,
  formatPercentTriplet,
  formatRoleLabel,
  formatTimestamp,
  formatVersionRevision,
} from '~/utils/metricsFormat';
import { calculateUptimePercentage } from '~/utils/metrics';
import { getStatusFromK8sStatus } from '~/utils/generic';

const textDecoder = new TextDecoder();

const metricDescriptions = {
  status: 'Current workload state reported by Kubernetes.',
  health: 'Percentage of the last 30 days this workload was healthy.',
  blockHeight: 'Latest block number observed by the node.',
  epoch: 'Current epoch observed by the node.',
  slotInEpoch: 'Current slot within the active epoch observed by the node.',
  epochProgress:
    'Percentage of the current epoch completed from slot-in-epoch and Shelley genesis epoch length.',
  epochTimeRemaining:
    'Approximate time remaining in the current epoch derived from Shelley genesis timing.',
  absoluteSlot:
    'Absolute slot number observed by the node across the chain timeline.',
  tipRefSlot:
    'Reference chain tip computed from the Shelley genesis system start and slot length.',
  tipDiffSlots:
    'Difference between the computed reference tip and the node tip.',
  syncPercent: 'Estimated sync percentage against the computed reference tip.',
  density:
    'Recent chain density reported by the node, expressed as a percentage.',
  forks: 'Number of chain forks the node has observed since startup.',
  nodeVersion:
    'Cardano node build version and revision reported by the metrics endpoint.',
  forgingEnabled: 'Whether this node currently has forging enabled.',
  pendingTx:
    'Transactions currently in the mempool, plus buffered size when available.',
  txProcessed: 'Total transactions processed by the node since startup.',
  peersInOut: 'Active inbound and outbound node connections.',
  connectionDirections:
    'Current connection counts split by unidirectional, bidirectional, and full duplex.',
  inboundStates: 'Inbound governor connection states reported by the node.',
  outboundStates: 'Peer selection states for outbound connections.',
  lastBlockDelay: 'Latest observed block propagation delay.',
  blocksServed: 'Blocks served to peers by this node since startup.',
  blocksLate:
    'Blocks observed later than five seconds by the block fetch client.',
  propagationCdf:
    'Percentage of observed blocks arriving within 1, 3, and 5 seconds.',
  memLive: 'Live RTS memory currently retained by the node process.',
  memHeap: 'Heap memory currently reserved by the node RTS.',
  gcMinor: 'Number of minor garbage collections since startup.',
  gcMajor: 'Number of major garbage collections since startup.',
  blocksAdopted: 'Blocks successfully adopted by this producer since startup.',
  scheduledLeader:
    'Scheduled leadership slots for the current epoch derived from cardano-cli leadership schedule.',
  scheduledIdeal:
    'Ideal expected block count for the current epoch derived from the current stake snapshot, active slots coefficient, and epoch length.',
  scheduledLuck:
    'Current epoch leader schedule luck, computed as scheduled leadership slots divided by the ideal expected slot count.',
  nextBlock:
    'Time remaining until the next scheduled leadership slot in the current epoch.',
  kesSummary:
    'Current KES period and how many periods remain before rotation is required.',
  opCertSummary:
    'Operational certificate counters seen on disk and in the node chain state.',
  kesExpiration:
    'Approximate KES expiration time derived from remaining KES periods and Shelley genesis timing.',
  kesExpirationRemaining:
    'Approximate time remaining before KES rotation is required.',
  leaderAdopted:
    'Leadership slots assigned to this producer versus blocks adopted.',
  forgedAboutToLead:
    'Forged blocks versus slots the node reports as about to lead.',
  invalidMissed:
    'Forged but not adopted blocks, and scheduled slots the node missed.',
} as const;

async function getWorkloadDetails(namespace: string, name: string) {
  const data = await getServerWorkloadPods({ data: { namespace, name } });

  if (data.error || !data.items?.length) {
    throw redirect({
      to: '/',
    });
  }

  const dashboardUrl = await getGrafanaDashboardUrl({
    data: { namespace },
  }).catch(err => {
    // eslint-disable-next-line no-console
    console.log(err);
    return null;
  });

  return {
    items: data.items,
    dashboardUrl,
  };
}

const workloadDetailsQueryOptions = (namespace: string, name: string) =>
  queryOptions({
    queryKey: ['workloadDetails', namespace, name],
    queryFn: () => getWorkloadDetails(namespace, name),
    refetchInterval: 5000,
    refetchIntervalInBackground: true,
  });

export const Route = createFileRoute('/$namespace/$name/')({
  loader: async ({ context, params }) => {
    await context.queryClient.ensureQueryData(
      workloadDetailsQueryOptions(params.namespace, params.name),
    );
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
      <div className={twMerge('text-sm text-[#2B2B2B]/80', valueClassName)}>
        {value}
      </div>
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
        <div className="text-xs font-semibold uppercase tracking-[0.16em] text-[#64748B]">
          {title}
        </div>
        {aside}
      </div>
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-6">
        {children}
      </div>
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
      toast.custom(t => (
        <Toast
          toastId={t.id}
          title="Workload Deleted"
          message="Your workload was deleted successfully."
          style="success"
        />
      ));
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
  const workloadDetailsQuery = useSuspenseQuery(
    workloadDetailsQueryOptions(namespace, name),
  );
  const { items, dashboardUrl } = workloadDetailsQuery.data;
  const logsContainerRef = useRef<HTMLDivElement>(null);
  const readableStreamRef = useRef<ReadableStreamDefaultReader<any> | null>(
    null,
  );
  const activePod = items[0];
  const activePodNamespace = activePod?.namespace;
  const activePodContainerName = activePod?.containerName;
  const activePodKey = [
    activePod?.name,
    activePodNamespace,
    activePodContainerName,
  ]
    .filter(Boolean)
    .join('/');
  const [logState, setLogState] = useState(() => ({
    podKey: activePodKey,
    value: '',
  }));
  const logs = logState.podKey === activePodKey ? logState.value : '';

  useEffect(() => {
    if (logsContainerRef.current) {
      logsContainerRef.current.scroll({
        top: logsContainerRef.current.scrollHeight,
        behavior: 'smooth',
      });
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
            value: (
              (prev.podKey === activePodKey ? prev.value : '') + text
            ).slice(-10000),
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
  }, [
    activePod?.name,
    activePodContainerName,
    activePodKey,
    activePodNamespace,
  ]);

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
          <span className="font-semibold text-[#2B2B2B]">
            {activePod.annotations?.displayName ?? activePod.containerName}
          </span>
        </div>

        <div className="flex flex-row items-start gap-4 mt-4">
          <img
            src={activePod.annotations?.icon}
            alt={`${activePod.annotations?.displayName ?? activePod.containerName} logo`}
            className="w-15.5 h-15.5"
          />
          <div className="grow">
            <h1 className="text-[32px] font-semibold text-[#2B2B2B]">
              {activePod.annotations?.displayName ?? activePod.containerName}
            </h1>
            <span className="mt-1 text-[#969FAB] leading-none">
              {activePod.annotations?.network}
            </span>
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
            aside={
              cardanoNodeMetrics && (
                <span className="rounded-full border border-zinc-200 bg-white px-3 py-1 text-xs font-medium text-[#64748B]">
                  {formatRoleLabel(cardanoNodeMetrics.role)}
                </span>
              )
            }
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
                  label="Epoch"
                  value={formatMetricValue(cardanoNodeMetrics.epoch)}
                  description={metricDescriptions.epoch}
                />
                <InfoCard
                  label="Slot in epoch"
                  value={formatMetricValue(cardanoNodeMetrics.slotInEpoch)}
                  description={metricDescriptions.slotInEpoch}
                />
                <InfoCard
                  label="Epoch progress"
                  value={formatPercent(cardanoNodeMetrics.epochProgressPercent)}
                  description={metricDescriptions.epochProgress}
                />
                <InfoCard
                  label="Epoch remaining"
                  value={formatDurationSeconds(
                    cardanoNodeMetrics.epochTimeRemainingSeconds,
                  )}
                  description={metricDescriptions.epochTimeRemaining}
                />
                <InfoCard
                  label="Absolute slot"
                  value={formatMetricValue(cardanoNodeMetrics.slotNum)}
                  description={metricDescriptions.absoluteSlot}
                />
                <InfoCard
                  label="Tip ref slot"
                  value={formatMetricValue(cardanoNodeMetrics.tipRefSlot)}
                  description={metricDescriptions.tipRefSlot}
                />
                <InfoCard
                  label="Tip diff"
                  value={formatMetricValue(cardanoNodeMetrics.tipDiffSlots)}
                  description={metricDescriptions.tipDiffSlots}
                />
                <InfoCard
                  label="Sync"
                  value={formatPercent(cardanoNodeMetrics.syncPercent)}
                  description={metricDescriptions.syncPercent}
                />
                <InfoCard
                  label="Density"
                  value={formatPercent(cardanoNodeMetrics.density)}
                  description={metricDescriptions.density}
                />
                <InfoCard
                  label="Forks"
                  value={formatMetricValue(cardanoNodeMetrics.forks)}
                  description={metricDescriptions.forks}
                />
                <InfoCard
                  label="Node version"
                  value={formatVersionRevision(
                    cardanoNodeMetrics.nodeVersion,
                    cardanoNodeMetrics.nodeRevision,
                  )}
                  description={metricDescriptions.nodeVersion}
                />
                <InfoCard
                  label="Forging enabled"
                  value={formatBooleanMetric(cardanoNodeMetrics.forgingEnabled)}
                  description={metricDescriptions.forgingEnabled}
                  valueClassName={clsx({
                    'text-[#69C876]':
                      cardanoNodeMetrics.forgingEnabled === true,
                    'text-[#FF7474]':
                      cardanoNodeMetrics.forgingEnabled === false,
                  })}
                />
              </>
            )}
          </MetricsSection>

          {cardanoNodeMetrics && (
            <>
              <MetricsSection title="Mempool" withDivider>
                <InfoCard
                  label="Pending tx"
                  value={formatPendingTx(
                    cardanoNodeMetrics.pendingTx,
                    cardanoNodeMetrics.pendingTxBytes,
                  )}
                  description={metricDescriptions.pendingTx}
                />
                <InfoCard
                  label="Tx processed"
                  value={formatMetricValue(cardanoNodeMetrics.txProcessed)}
                  description={metricDescriptions.txProcessed}
                />
              </MetricsSection>

              <MetricsSection title="Connections" withDivider>
                <InfoCard
                  label="Peers in / out"
                  value={formatPeerCounts(
                    cardanoNodeMetrics.peersIncoming,
                    cardanoNodeMetrics.peersOutgoing,
                  )}
                  description={metricDescriptions.peersInOut}
                />
                <InfoCard
                  label="Uni / bi / duplex"
                  value={formatCountTriplet(
                    cardanoNodeMetrics.connectionUniDir,
                    cardanoNodeMetrics.connectionBiDir,
                    cardanoNodeMetrics.connectionDuplex,
                  )}
                  description={metricDescriptions.connectionDirections}
                />
                <InfoCard
                  label="Inbound warm / hot"
                  value={formatCountPair(
                    cardanoNodeMetrics.inboundGovernorWarm,
                    cardanoNodeMetrics.inboundGovernorHot,
                  )}
                  description={metricDescriptions.inboundStates}
                />
                <InfoCard
                  label="Out cold / warm / hot"
                  value={formatCountTriplet(
                    cardanoNodeMetrics.peerSelectionCold,
                    cardanoNodeMetrics.peerSelectionWarm,
                    cardanoNodeMetrics.peerSelectionHot,
                  )}
                  description={metricDescriptions.outboundStates}
                />
              </MetricsSection>

              <MetricsSection title="Block propagation" withDivider>
                <InfoCard
                  label="Last block delay"
                  value={formatDelaySeconds(
                    cardanoNodeMetrics.lastBlockDelaySeconds,
                  )}
                  description={metricDescriptions.lastBlockDelay}
                />
                <InfoCard
                  label="Served"
                  value={formatMetricValue(cardanoNodeMetrics.blocksServed)}
                  description={metricDescriptions.blocksServed}
                />
                <InfoCard
                  label="Late (>5s)"
                  value={formatMetricValue(cardanoNodeMetrics.blocksLate)}
                  description={metricDescriptions.blocksLate}
                />
                <InfoCard
                  label="Within 1 / 3 / 5s"
                  value={formatPercentTriplet(
                    cardanoNodeMetrics.blocksWithin1s,
                    cardanoNodeMetrics.blocksWithin3s,
                    cardanoNodeMetrics.blocksWithin5s,
                  )}
                  description={metricDescriptions.propagationCdf}
                />
              </MetricsSection>

              <MetricsSection title="Resources" withDivider>
                <InfoCard
                  label="Mem live"
                  value={formatBytesToGiB(cardanoNodeMetrics.memLiveBytes)}
                  description={metricDescriptions.memLive}
                />
                <InfoCard
                  label="Mem heap"
                  value={formatBytesToGiB(cardanoNodeMetrics.memHeapBytes)}
                  description={metricDescriptions.memHeap}
                />
                <InfoCard
                  label="GC minor"
                  value={formatMetricValue(cardanoNodeMetrics.gcMinorCount)}
                  description={metricDescriptions.gcMinor}
                />
                <InfoCard
                  label="GC major"
                  value={formatMetricValue(cardanoNodeMetrics.gcMajorCount)}
                  description={metricDescriptions.gcMajor}
                />
              </MetricsSection>

              {isBlockProducer && (
                <MetricsSection title="Producer" withDivider>
                  <InfoCard
                    label="Leader"
                    value={formatMetricValue(
                      cardanoNodeMetrics.scheduledLeaderCount,
                    )}
                    description={metricDescriptions.scheduledLeader}
                  />
                  <InfoCard
                    label="Ideal"
                    value={formatMetricValue(
                      cardanoNodeMetrics.scheduledIdealCount,
                      { maximumFractionDigits: 2 },
                    )}
                    description={metricDescriptions.scheduledIdeal}
                  />
                  <InfoCard
                    label="Luck"
                    value={formatPercent(
                      cardanoNodeMetrics.scheduledLuckPercent,
                    )}
                    description={metricDescriptions.scheduledLuck}
                  />
                  <InfoCard
                    label="Next Block in"
                    value={formatDurationSeconds(
                      cardanoNodeMetrics.nextLeaderTimeRemainingSeconds,
                    )}
                    description={metricDescriptions.nextBlock}
                  />
                  <InfoCard
                    label="KES current / remaining"
                    value={formatKesSummary(
                      cardanoNodeMetrics.kesPeriod,
                      cardanoNodeMetrics.kesRemaining,
                    )}
                    description={metricDescriptions.kesSummary}
                  />
                  <InfoCard
                    label="OP Cert disk | chain"
                    value={formatCountPair(
                      cardanoNodeMetrics.opCertOnDisk,
                      cardanoNodeMetrics.opCertOnChain,
                    )}
                    description={metricDescriptions.opCertSummary}
                  />
                  <InfoCard
                    label="KES expiration"
                    value={formatTimestamp(
                      cardanoNodeMetrics.kesExpirationTime,
                    )}
                    description={metricDescriptions.kesExpiration}
                  />
                  <InfoCard
                    label="KES expires in"
                    value={formatDurationSeconds(
                      cardanoNodeMetrics.kesExpirationSeconds,
                    )}
                    description={metricDescriptions.kesExpirationRemaining}
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
              dangerouslySetInnerHTML={{
                __html: converter.toHtml(logs || 'No logs available').trim(),
              }}
            />
          </div>
        </Card>
      </div>

      <Card className="flex flex-row items-center justify-between">
        <div>
          <CardTitle>Delete workload</CardTitle>
          <p className="mt-3 text-[13px]/none text-[#2B2B2B]">
            Once you delete a workload, there is no going back. Please be
            certain.
          </p>
        </div>

        <DeleteAction />
      </Card>
    </div>
  );
}
