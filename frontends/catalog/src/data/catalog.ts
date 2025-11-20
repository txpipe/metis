import { createServerFn } from '@tanstack/react-start';
import slugify from 'slugify';
import { getRepoInfo } from '~/utils/registry';

const items: CatalogItem[] = [
  {
    icon: '/images/workloads/midnight.svg',
    name: 'Midnight Node',
    slug: slugify('Midnight Node').toLowerCase(),
    description: 'A 4th generation blockchain node with focus on privacy. Become a Midnight validator.',
    category: 'partner-chain',
    comingSoon: false,
    author: {
      name: 'midnightntwrk',
      url: 'https://github.com/midnightntwrk',
    },
    repoUrl: 'https://github.com/midnightntwrk/midnight-node',
    version: '0.12.0',
    ociName: 'midnight',
    publishedDate: '2025-09-10',
  },
  {
    icon: '/images/workloads/cardano-node.svg',
    name: 'Cardano Node',
    slug: slugify('Cardano Node').toLowerCase(),
    description: 'Run a Cardano layer 1 node to validate blocks and strengthen the network.',
    category: 'layer-1',
    comingSoon: false,
    beta: true,
  },
  {
    icon: '/images/workloads/apex-fusion.png',
    name: 'Apex Fusion Prime Node',
    slug: slugify('Apex Fusion Prime Node').toLowerCase(),
    description: 'The foundational UTXO-based layer 1 for the Apex Fusion\'s ecosystem. Run a Prime node.',
    category: 'layer-1',
    comingSoon: false,
    beta: true,
  },
  {
    icon: '/images/workloads/hydra.svg',
    name: 'Hydra Node',
    slug: slugify('Hydra Node').toLowerCase(),
    description: 'The layer 2 scalability solution for Cardano. Implements the Hydra Head protocol.',
    category: 'layer-2',
    comingSoon: false,
    beta: true,
  },
  {
    icon: '/images/workloads/dolos.svg',
    name: 'Dolos',
    slug: slugify('Dolos').toLowerCase(),
    description: 'Lorem ipsum dolor sit amet consectetur. Condimentum vitae sit fringilla at nisl.',
    category: 'infrastructure',
    comingSoon: false,
    beta: true,
  },
  {
    icon: '/images/workloads/midgard.png',
    name: 'Midgard Node',
    slug: slugify('Midgard Node').toLowerCase(),
    description: 'The first optimistic rollup protocol on Cardano. Become a Midgard operator.',
    category: 'layer-2',
    comingSoon: true,
  },
  {
    icon: '/images/workloads/sundial.svg',
    name: 'Sundial Node',
    slug: slugify('Sundial Node').toLowerCase(),
    description: 'A layer 2 on Cardano custom built to serve as Bitcoin\'s utility and yield layer.',
    category: 'layer-2',
    comingSoon: true,
  },
  {
    icon: '/images/workloads/orcfax-oracle.png',
    name: 'Orcfax Oracle',
    slug: slugify('Orcfax Oracle').toLowerCase(),
    description: 'Run an Orcfax node to publish verified, fact-based data on-chain.',
    category: 'oracle',
    comingSoon: true,
  },
  {
    icon: '/images/workloads/butane-oracle.png',
    name: 'Butane Oracle',
    slug: slugify('Butane Oracle').toLowerCase(),
    description: 'Run a Butane oracle node to provide decentralized, signer-based data feeds.',
    category: 'oracle',
    comingSoon: true,
  },
  {
    icon: '/images/workloads/fluidtokens.png',
    name: 'Aquarium Node',
    slug: slugify('Aquarium Node').toLowerCase(),
    description: 'Support Babel fees on Cardano by running Aquarium Node — by FluidTokens.',
    category: 'batcher',
    comingSoon: true,
  },
  {
    icon: '/images/workloads/fluidtokens.png',
    name: 'Bifrost Bridge',
    slug: slugify('Bifrost Bridge').toLowerCase(),
    description: 'Operate a node to support a Bitcoin to Cardano secured bridge — by FluidTokens.',
    category: 'bridge',
    comingSoon: true,
  },
  {
    icon: '/images/workloads/sundae-swap.png',
    name: 'Sundae Swap Scooper',
    slug: slugify('Sundae Swap Scooper').toLowerCase(),
    description: 'Lorem ipsum dolor sit amet consectetur. Condimentum vitae sit fringilla at nisl.',
    category: 'batcher',
    comingSoon: true,
  },
  {
    icon: '/images/workloads/minswap.png',
    name: 'Minswap Batcher',
    slug: slugify('Minswap Batcher').toLowerCase(),
    description: 'Run a Minswap batcher to help process and finalize DEX operations.',
    category: 'batcher',
    comingSoon: true,
  },
  {
    icon: '/images/workloads/deltadefi.png',
    name: 'Deltadefi Node',
    slug: slugify('Deltadefi Node').toLowerCase(),
    description: 'Operate a DeltaDeFi\'s Hydra Node to support high-throughput L2 execution.',
    category: 'layer-2',
    comingSoon: true,
  },
  {
    icon: '/images/workloads/demeter.svg',
    name: 'Demeter DePIN Node',
    slug: slugify('Demeter DePIN Node').toLowerCase(),
    description: 'Lorem ipsum dolor sit amet consectetur. Condimentum vitae sit fringilla at nisl.',
    category: 'partner-chain',
    comingSoon: true,
  },
  {
    icon: '/images/workloads/quantum-hosky.svg',
    name: 'Quantum Hosky Node',
    slug: slugify('Quantum Hosky Node').toLowerCase(),
    description: 'A layered metaverse partner chain. Become a Quantum Hosky block producer.',
    category: 'partner-chain',
    comingSoon: true,
  },
  {
    icon: '/images/workloads/blockfrost-icebreaker.svg',
    name: 'Blockfrost Icebreaker',
    slug: slugify('Blockfrost Icebreaker').toLowerCase(),
    description: 'Help decentralize the Blockfrost API, by running the IceBreaker node.',
    category: 'partner-chain',
    comingSoon: true,
  },
];

export const getCatalog = createServerFn({
  method: 'GET',
}).handler(async (): Promise<CatalogItem[]> => {
  return items;
});

export const getItemBySlug = createServerFn({
  method: 'GET',
})
  .inputValidator((data: { slug: string; }) => data)
  .handler(async ({ data }): Promise<CatalogItem | undefined> => {
    const item = items.find(i => i.slug === data.slug);

    if (item?.ociName) {
      const repoInfo = await getRepoInfo(`extensions/${item.ociName}`);
      if (repoInfo.data?.ExpandedRepoInfo) {
        item.registryInfo = repoInfo.data.ExpandedRepoInfo;
      }
    }
    return item;
  });
