export function formatMetricValue(value: number | null | undefined, options?: Intl.NumberFormatOptions) {
  if (value === null || value === undefined || Number.isNaN(value)) {
    return '-';
  }

  return new Intl.NumberFormat('en-US', options).format(value);
}

export function formatDelaySeconds(value: number | null | undefined) {
  if (value === null || value === undefined) {
    return '-';
  }

  return `${formatMetricValue(value, { maximumFractionDigits: 2 })}s`;
}

export function formatBytes(value: number | null | undefined) {
  if (value === null || value === undefined) {
    return '-';
  }

  const kibibytes = value / 1024;
  if (kibibytes >= 1024) {
    return `${formatMetricValue(kibibytes / 1024, { maximumFractionDigits: 1 })} MiB`;
  }

  return `${formatMetricValue(kibibytes, { maximumFractionDigits: 1 })} KiB`;
}

export function formatBytesToGiB(value: number | null | undefined) {
  if (value === null || value === undefined) {
    return '-';
  }

  return `${formatMetricValue(value / (1024 ** 3), { maximumFractionDigits: 1 })} GiB`;
}

export function formatPendingTx(count: number | null | undefined, bytes: number | null | undefined) {
  if (count === null || count === undefined) {
    return '-';
  }

  if (bytes === null || bytes === undefined) {
    return `${formatMetricValue(count)} tx`;
  }

  return `${formatMetricValue(count)} tx / ${formatBytes(bytes)}`;
}

export function formatPeerCounts(incoming: number | null | undefined, outgoing: number | null | undefined) {
  if ((incoming === null || incoming === undefined) && (outgoing === null || outgoing === undefined)) {
    return '-';
  }

  return `${formatMetricValue(incoming)} / ${formatMetricValue(outgoing)}`;
}

export function formatEpochSlot(epoch: number | null | undefined, slotInEpoch: number | null | undefined) {
  if ((epoch === null || epoch === undefined) && (slotInEpoch === null || slotInEpoch === undefined)) {
    return '-';
  }

  return `E${formatMetricValue(epoch)} / S${formatMetricValue(slotInEpoch)}`;
}

export function formatKesSummary(kesPeriod: number | null | undefined, kesRemaining: number | null | undefined) {
  if ((kesPeriod === null || kesPeriod === undefined) && (kesRemaining === null || kesRemaining === undefined)) {
    return '-';
  }

  return `${formatMetricValue(kesPeriod)} / ${formatMetricValue(kesRemaining)} left`;
}

export function formatCountPair(primary: number | null | undefined, secondary: number | null | undefined) {
  if ((primary === null || primary === undefined) && (secondary === null || secondary === undefined)) {
    return '-';
  }

  return `${formatMetricValue(primary)} / ${formatMetricValue(secondary)}`;
}

export function formatCountTriplet(
  first: number | null | undefined,
  second: number | null | undefined,
  third: number | null | undefined,
) {
  if (
    (first === null || first === undefined)
    && (second === null || second === undefined)
    && (third === null || third === undefined)
  ) {
    return '-';
  }

  return `${formatMetricValue(first)} / ${formatMetricValue(second)} / ${formatMetricValue(third)}`;
}

export function formatPercent(value: number | null | undefined) {
  if (value === null || value === undefined) {
    return '-';
  }

  return `${formatMetricValue(value, { maximumFractionDigits: 2 })}%`;
}

export function formatPercentTriplet(
  first: number | null | undefined,
  second: number | null | undefined,
  third: number | null | undefined,
) {
  if (
    (first === null || first === undefined)
    && (second === null || second === undefined)
    && (third === null || third === undefined)
  ) {
    return '-';
  }

  return `${formatPercent(first)} / ${formatPercent(second)} / ${formatPercent(third)}`;
}

export function formatBooleanMetric(value: boolean | null | undefined) {
  if (value === null || value === undefined) {
    return '-';
  }

  return value ? 'Enabled' : 'Disabled';
}

export function formatVersionRevision(version: string | null | undefined, revision: string | null | undefined) {
  if (!version && !revision) {
    return '-';
  }

  if (!revision) {
    return version ?? '-';
  }

  if (!version) {
    return revision;
  }

  return `${version} [${revision}]`;
}

export function formatRoleLabel(role: NodeRole) {
  return role === 'block-producer' ? 'Block producer' : 'Relay';
}
