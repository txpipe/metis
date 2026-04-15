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

function getPodSelector(namespace: string, name: string) {
  return `namespace="${namespace}", pod=~"${name}-[0-9]+"`;
}

async function getInstantMetricValue(query: string): Promise<number | null> {
  try {
    const res = await driver.instantQuery(query);
    const value = res.result?.[0]?.value?.value;

    if (value === undefined || value === null) {
      return null;
    }

    const parsedValue = Number(value);
    return Number.isFinite(parsedValue) ? parsedValue : null;
  } catch (_) {
    return null;
  }
}

async function getFirstMatchingMetricValue(queries: string[]): Promise<number | null> {
  for (const query of queries) {
    const value = await getInstantMetricValue(query);
    if (value !== null) {
      return value;
    }
  }

  return null;
}

function wrapMetricQuery(metricName: string, selector: string) {
  return `max(${metricName}{${selector}})`;
}

export async function getCardanoNodeMetrics(
  namespace: string,
  name: string,
  role: NodeRole,
): Promise<CardanoNodeMetrics> {
  const selector = getPodSelector(namespace, name);

  const [
    blockHeight,
    epoch,
    slotInEpoch,
    density,
    txProcessed,
    pendingTx,
    pendingTxBytes,
    peersIncoming,
    peersOutgoing,
    lastBlockDelaySeconds,
    kesPeriod,
    kesRemaining,
    leaderCount,
    adoptedCount,
    forgedCount,
    missedSlots,
  ] = await Promise.all([
    getFirstMatchingMetricValue([
      wrapMetricQuery('cardano_node_metrics_blockNum_int', selector),
    ]),
    getFirstMatchingMetricValue([
      wrapMetricQuery('cardano_node_metrics_epoch_int', selector),
    ]),
    getFirstMatchingMetricValue([
      wrapMetricQuery('cardano_node_metrics_slotInEpoch_int', selector),
    ]),
    getFirstMatchingMetricValue([
      wrapMetricQuery('cardano_node_metrics_density_real', selector),
    ]),
    getFirstMatchingMetricValue([
      wrapMetricQuery('cardano_node_metrics_txsProcessedNum_int', selector),
      wrapMetricQuery('cardano_node_metrics_txsProcessedNum_counter', selector),
    ]),
    getFirstMatchingMetricValue([
      wrapMetricQuery('cardano_node_metrics_txsInMempool_int', selector),
    ]),
    getFirstMatchingMetricValue([
      wrapMetricQuery('cardano_node_metrics_mempoolBytes_int', selector),
    ]),
    getFirstMatchingMetricValue([
      wrapMetricQuery('cardano_node_metrics_connectionManager_incomingConns_int', selector),
      wrapMetricQuery('cardano_node_metrics_connectionManager_inboundConns_int', selector),
      wrapMetricQuery('cardano_node_metrics_connectionManager_incomingConns', selector),
      wrapMetricQuery('cardano_node_metrics_connectionManager_inboundConns', selector),
    ]),
    getFirstMatchingMetricValue([
      wrapMetricQuery('cardano_node_metrics_connectionManager_outgoingConns_int', selector),
      wrapMetricQuery('cardano_node_metrics_connectionManager_outboundConns_int', selector),
      wrapMetricQuery('cardano_node_metrics_connectionManager_outgoingConns', selector),
      wrapMetricQuery('cardano_node_metrics_connectionManager_outboundConns', selector),
    ]),
    getFirstMatchingMetricValue([
      wrapMetricQuery('cardano_node_metrics_blockfetchclient_blockdelay_s', selector),
      wrapMetricQuery('cardano_node_metrics_blockfetchclient_blockdelay_real', selector),
    ]),
    getFirstMatchingMetricValue([
      wrapMetricQuery('cardano_node_metrics_currentKESPeriod_int', selector),
    ]),
    getFirstMatchingMetricValue([
      wrapMetricQuery('cardano_node_metrics_remainingKESPeriods_int', selector),
    ]),
    getFirstMatchingMetricValue([
      wrapMetricQuery('cardano_node_metrics_Forge_node_is_leader_int', selector),
      wrapMetricQuery('cardano_node_metrics_Forge_node_is_leader_counter', selector),
    ]),
    getFirstMatchingMetricValue([
      wrapMetricQuery('cardano_node_metrics_Forge_adopted_int', selector),
      wrapMetricQuery('cardano_node_metrics_Forge_adopted_counter', selector),
    ]),
    getFirstMatchingMetricValue([
      wrapMetricQuery('cardano_node_metrics_Forge_forged_int', selector),
      wrapMetricQuery('cardano_node_metrics_Forge_forged_counter', selector),
    ]),
    getFirstMatchingMetricValue([
      wrapMetricQuery('cardano_node_metrics_slotsMissedNum_int', selector),
      wrapMetricQuery('cardano_node_metrics_slotsMissed_int', selector),
    ]),
  ]);

  const invalidCount = (forgedCount !== null && adoptedCount !== null)
    ? Math.max(forgedCount - adoptedCount, 0)
    : null;

  return {
    type: 'cardano-node',
    role,
    blockHeight,
    epoch,
    slotInEpoch,
    density: density !== null ? density * 100 : null,
    txProcessed,
    pendingTx,
    pendingTxBytes,
    peersIncoming,
    peersOutgoing,
    lastBlockDelaySeconds,
    kesPeriod,
    kesRemaining,
    leaderCount,
    adoptedCount,
    invalidCount,
    missedSlots,
  };
}
