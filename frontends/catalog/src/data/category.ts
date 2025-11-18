import type { Icons } from '~/components/icons';

export const categories: { icon: Icons; label: string; value: string; }[] = [
  { icon: 'TopologyRingIcon', label: 'L1 Nodes', value: 'layer-1' },
  { icon: 'ChartCirclesIcon', label: 'Partner-chains', value: 'partner-chain' },
  { icon: 'Stack2Icon', label: 'L2 Nodes', value: 'layer-2' },
  { icon: 'BuildingBridgeIcon', label: 'Bridges', value: 'bridge' },
  { icon: 'InnerShadowTopLeftIcon', label: 'Oracles', value: 'oracle' },
  { icon: 'BoxIcon', label: 'Batchers', value: 'batcher' },
  { icon: 'Settings2Icon', label: 'Infrastructure', value: 'infrastructure' },
];

export function getCategoryLabel(value: string): string | undefined {
  const category = categories.find(c => c.value === value);
  return category?.label;
}
