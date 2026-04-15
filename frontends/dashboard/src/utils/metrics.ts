import { PrometheusDriver, SampleValue } from 'prometheus-query';
import dayjs from 'dayjs';
import utc from 'dayjs/plugin/utc';
dayjs.extend(utc);

const driver = new PrometheusDriver({ endpoint: process.env.PROMETHEUS_ENDPOINT || 'http://localhost:9090' });

export const emptyUptimeResult: UptimeEntry[] = Array.from({ length: 30 }, (_, i) => {
  const date = dayjs.utc().subtract(29 - i, 'day').format('YYYY-MM-DD');
  return {
    date,
    uptimeHours: 0,
    state: -1,
  };
});

// Minimum hours to consider a day as "up"
const THRESHOLD_HOURS_UP = 0;
export async function getStatefulSetUptime(namespace: string, name: string, fillSize = 30): Promise<UptimeEntry[]> {
  try {
    const commonParams = `namespace="${namespace}", pod=~"${name}-[0-9]+"`;

    const uptimeQuery = `
      sum_over_time(
        clamp_max(
          sum(kube_pod_status_ready{${commonParams}, condition="true"}), 1
        )[1d:1h]
      )
    `;

    const startQuery = `kube_statefulset_created{namespace="${namespace}", statefulset="${name}"}`;
    const startRes = await driver.instantQuery(startQuery);
    const podStartTime = parseFloat(startRes.result?.[0]?.value?.value || '0');

    const end = dayjs.utc().endOf('day');
    const start = dayjs.utc().subtract(30, 'day').startOf('day');

    const res = await driver.rangeQuery(uptimeQuery, start.toDate(), end.toDate(), 86400);

    const values: SampleValue[] = res.result[0]?.values;

    let firstDayWithData: dayjs.Dayjs | null = null;

    let result = values?.map((item, idx) => {
      const time = dayjs(item.time);
      if (idx === 0) {
        firstDayWithData = time;
      }
      const uptimeHours = item.value; // Número de horas up
      let state = 0; // with downtime

      if (time.unix() < podStartTime) { // Didn't exist
        state = -1;
      } else if (uptimeHours >= THRESHOLD_HOURS_UP) { // Tolerancia: <1 hora de downtime se considera up
        state = 1;
      }

      return {
        date: time.format('YYYY-MM-DD'),
        uptimeHours,
        state, // -1 | 0 | 1
      };
    }) ?? [];

    const today = dayjs.utc().startOf('day');
    const now = dayjs.utc();
    const todayStartMs = today.valueOf();
    const nowMs = now.valueOf();
    const todayQuery = `
      clamp_max(
        sum(kube_pod_status_ready{${commonParams}, condition="true"}), 1
      )
    `;
    const todayRes = await driver.rangeQuery(todayQuery, todayStartMs, nowMs, 3600);
    const todayUptimeHours = todayRes.result[0]?.values?.reduce(
      (sum: number, item: SampleValue) => sum + item.value, 0,
    ) || 0;
    // const hoursElapsed = now.diff(today, 'hour');
    result.push({
      date: today.format('YYYY-MM-DD'),
      uptimeHours: todayUptimeHours,
      // state: todayUptimeHours >= hoursElapsed - 1 ? 1 : 0,
      state: 1,
    });

    const missingDays = fillSize - result.length;
    if (missingDays <= 0) {
      return result.slice(-fillSize);
    }

    if (!firstDayWithData) {
      firstDayWithData = dayjs().utc();
    }

    for (let i = 0; i < missingDays; i++) {
      const date = firstDayWithData.clone().subtract(i + 1, 'day').format('YYYY-MM-DD');
      result.unshift({
        date,
        uptimeHours: 0,
        state: -1,
      });
    }

    return result;
  } catch (_) {
    return emptyUptimeResult;
  }
}

export function calculateUptimePercentage(uptimeData?: UptimeEntry[]): number {
  if (!uptimeData) return 0;
  const validDays = uptimeData.filter(entry => entry.state !== -1); // Días con datos
  if (validDays.length === 0) return 0;

  const healthyDays = validDays.filter(entry => entry.state === 1).length; // Días con uptime completo
  const percentage = (healthyDays / validDays.length) * 100;
  return Math.round(percentage * 100) / 100; // Redondear a 2 decimales
}

function readMetricValue(metricsText: string, regex: RegExp): number | null {
  const match = metricsText.match(regex);
  if (!match) {
    return null;
  }

  const rawValue = match[match.length - 1];
  if (!rawValue) {
    return null;
  }

  const parsedValue = Number(rawValue);
  return Number.isFinite(parsedValue) ? parsedValue : null;
}

function readFirstMatchingMetricValue(metricsText: string, regexes: RegExp[]): number | null {
  for (const regex of regexes) {
    const value = readMetricValue(metricsText, regex);
    if (value !== null) {
      return value;
    }
  }

  return null;
}

function readBuildInfo(metricsText: string) {
  const versionMatch = metricsText.match(/cardano_node_metrics_cardano_build_info[^\n]*version="([^"]+)"/);
  const revisionMatch = metricsText.match(/cardano_node_metrics_cardano_build_info[^\n]*revision="([^"]+)"/);

  return {
    version: versionMatch?.[1] ?? null,
    revision: revisionMatch?.[1] ? revisionMatch[1].slice(0, 8) : null,
  };
}

export function parseCardanoNodeMetrics(metricsText: string, role: NodeRole): CardanoNodeMetrics {
  const blockHeight = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_blockNum_int[\s]+([^\s]+)/,
  ]);
  const epoch = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_epoch_int[\s]+([^\s]+)/,
  ]);
  const slotNum = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_slotNum_int[\s]+([^\s]+)/,
  ]);
  const slotInEpoch = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_slotInEpoch_int[\s]+([^\s]+)/,
  ]);
  const density = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_density_real[\s]+([^\s]+)/,
  ]);
  const forks = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_forks_(?:int|counter)[\s]+([^\s]+)/,
  ]);
  const txProcessed = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_txsProcessedNum_int[\s]+([^\s]+)/,
    /cardano_node_metrics_txsProcessedNum_counter[\s]+([^\s]+)/,
  ]);
  const pendingTx = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_txsInMempool_int[\s]+([^\s]+)/,
  ]);
  const pendingTxBytes = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_mempoolBytes_int[\s]+([^\s]+)/,
  ]);
  const peersIncoming = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_connectionManager_incomingConns_int[\s]+([^\s]+)/,
    /cardano_node_metrics_connectionManager_inboundConns_int[\s]+([^\s]+)/,
    /cardano_node_metrics_connectionManager_incomingConns[\s]+([^\s]+)/,
    /cardano_node_metrics_connectionManager_inboundConns[\s]+([^\s]+)/,
  ]);
  const peersOutgoing = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_connectionManager_outgoingConns_int[\s]+([^\s]+)/,
    /cardano_node_metrics_connectionManager_outboundConns_int[\s]+([^\s]+)/,
    /cardano_node_metrics_connectionManager_outgoingConns[\s]+([^\s]+)/,
    /cardano_node_metrics_connectionManager_outboundConns[\s]+([^\s]+)/,
  ]);
  const connectionUniDir = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_connectionManager_unidirectionalConns(?:_int|)[\s]+([^\s]+)/,
  ]);
  const connectionBiDir = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_connectionManager_duplexConns(?:_int|)[\s]+([^\s]+)/,
  ]);
  const connectionDuplex = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_connectionManager_fullDuplexConns(?:_int|)[\s]+([^\s]+)/,
  ]);
  const inboundGovernorWarm = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_inboundGovernor_warm(?:_int|)[\s]+([^\s]+)/,
  ]);
  const inboundGovernorHot = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_inboundGovernor_hot(?:_int|)[\s]+([^\s]+)/,
  ]);
  const peerSelectionCold = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_peerSelection_[cC]old(?:_int|)[\s]+([^\s]+)/,
  ]);
  const peerSelectionWarm = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_peerSelection_[wW]arm(?:_int|)[\s]+([^\s]+)/,
  ]);
  const peerSelectionHot = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_peerSelection_[hH]ot(?:_int|)[\s]+([^\s]+)/,
  ]);
  const lastBlockDelaySeconds = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_blockfetchclient_blockdelay_s[\s]+([^\s]+)/,
    /cardano_node_metrics_blockfetchclient_blockdelay_real[\s]+([^\s]+)/,
  ]);
  const blocksServed = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_served_block_count(?:er|_int)[\s]+([^\s]+)/,
  ]);
  const blocksLate = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_blockfetchclient_lateblocks(?:_counter|)[\s]+([^\s]+)/,
  ]);
  const blocksWithin1s = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_blockfetchclient_blockdelay_cdfOne(?:_real|)[\s]+([^\s]+)/,
  ]);
  const blocksWithin3s = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_blockfetchclient_blockdelay_cdfThree(?:_real|)[\s]+([^\s]+)/,
  ]);
  const blocksWithin5s = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_blockfetchclient_blockdelay_cdfFive(?:_real|)[\s]+([^\s]+)/,
  ]);
  const memLiveBytes = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_RTS_gcLiveBytes_int[\s]+([^\s]+)/,
  ]);
  const memHeapBytes = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_RTS_gcHeapBytes_int[\s]+([^\s]+)/,
  ]);
  const gcMinorCount = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_RTS_gcMinorNum_int[\s]+([^\s]+)/,
  ]);
  const gcMajorCount = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_RTS_gcMajorNum_int[\s]+([^\s]+)/,
  ]);
  const kesPeriod = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_currentKESPeriod_int[\s]+([^\s]+)/,
  ]);
  const kesRemaining = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_remainingKESPeriods_int[\s]+([^\s]+)/,
  ]);
  const leaderCount = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_Forge_node_is_leader_int[\s]+([^\s]+)/,
    /cardano_node_metrics_Forge_node_is_leader_counter[\s]+([^\s]+)/,
  ]);
  const adoptedCount = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_Forge_adopted_int[\s]+([^\s]+)/,
    /cardano_node_metrics_Forge_adopted_counter[\s]+([^\s]+)/,
  ]);
  const forgedCount = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_Forge_forged_int[\s]+([^\s]+)/,
    /cardano_node_metrics_Forge_forged_counter[\s]+([^\s]+)/,
  ]);
  const aboutToLeadCount = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_Forge_forge_about_to_lead_int[\s]+([^\s]+)/,
    /cardano_node_metrics_Forge_forge_about_to_lead_counter[\s]+([^\s]+)/,
    /cardano_node_metrics_Forge_about_to_lead_int[\s]+([^\s]+)/,
    /cardano_node_metrics_Forge_about_to_lead_counter[\s]+([^\s]+)/,
  ]);
  const missedSlots = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_slotsMissedNum_int[\s]+([^\s]+)/,
    /cardano_node_metrics_slotsMissed_int[\s]+([^\s]+)/,
  ]);
  const forgingEnabledMetric = readFirstMatchingMetricValue(metricsText, [
    /cardano_node_metrics_forging_enabled_int[\s]+([^\s]+)/,
    /cardano_node_metrics_forging_enabled[\s]+([^\s]+)/,
  ]);
  const { version: nodeVersion, revision: nodeRevision } = readBuildInfo(metricsText);

  const invalidCount = (forgedCount !== null && adoptedCount !== null)
    ? Math.max(forgedCount - adoptedCount, 0)
    : null;

  return {
    type: 'cardano-node',
    role,
    blockHeight,
    epoch,
    slotNum,
    slotInEpoch,
    density: density !== null ? density * 100 : null,
    forks,
    txProcessed,
    pendingTx,
    pendingTxBytes,
    nodeVersion,
    nodeRevision,
    forgingEnabled: forgingEnabledMetric !== null ? forgingEnabledMetric === 1 : null,
    peersIncoming,
    peersOutgoing,
    connectionUniDir,
    connectionBiDir,
    connectionDuplex,
    inboundGovernorWarm,
    inboundGovernorHot,
    peerSelectionCold,
    peerSelectionWarm,
    peerSelectionHot,
    lastBlockDelaySeconds,
    blocksServed,
    blocksLate,
    blocksWithin1s: blocksWithin1s !== null ? blocksWithin1s * 100 : null,
    blocksWithin3s: blocksWithin3s !== null ? blocksWithin3s * 100 : null,
    blocksWithin5s: blocksWithin5s !== null ? blocksWithin5s * 100 : null,
    memLiveBytes,
    memHeapBytes,
    gcMinorCount,
    gcMajorCount,
    kesPeriod,
    kesRemaining,
    leaderCount,
    adoptedCount,
    forgedCount,
    aboutToLeadCount,
    invalidCount,
    missedSlots,
  };
}
