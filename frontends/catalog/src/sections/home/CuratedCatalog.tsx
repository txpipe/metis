import { useNavigate } from '@tanstack/react-router';
import { useCallback } from 'react';

// Components
import { SearchInput } from '~/components/SearchInput';
import { Section } from '~/components/Section';

// Catalog route
import { Route as CatalogRoute } from '~/routes/catalog';

export function CuratedCatalogSection() {
  const navigate = useNavigate({ from: CatalogRoute.fullPath });

  const handleSearchText = useCallback((text: string | null) => {
    navigate({ to: '/catalog', search: prev => ({ ...prev, query: !!text ? text : undefined }), replace: true });
  }, [navigate]);

  return (
    <Section
      className="bg-zinc-100"
      sideBySide
    >
      <div className="flex md:-order-1 flex-wrap gap-6 justify-center opacity-89">
        <img src="/images/workloads-gray/midnight.svg" alt="Midnight logo" className="size-15.5" />
        <img src="/images/workloads-gray/midgard.png" alt="Midgard logo" className="size-15.5" />
        <img src="/images/workloads-gray/hydra.svg" alt="Hydra logo" className="size-15.5" />
        <img src="/images/workloads-gray/sundial.svg" alt="Sundial logo" className="size-15.5" />
        <img src="/images/workloads-gray/cardano-node.svg" alt="Cardano Node logo" className="size-15.5" />
        <img src="/images/workloads-gray/apex-fusion.png" alt="Apex Fusion logo" className="size-15.5" />
        <img src="/images/workloads-gray/orcfax-oracle.png" alt="Orcfax Oracle logo" className="size-15.5" />
        <img src="/images/workloads-gray/butane-oracle.png" alt="Butane Oracle logo" className="size-15.5" />
        <img src="/images/workloads-gray/sundae-swap.png" alt="Sundae Swap logo" className="size-15.5" />
        <img src="/images/workloads-gray/minswap.png" alt="Minswap logo" className="size-15.5" />
        <img src="/images/workloads-gray/demeter.svg" alt="Demeter logo" className="size-15.5" />
        <img src="/images/workloads-gray/dolos.svg" alt="Dolos logo" className="size-15.5" />

        <img src="/images/workloads-gray/blockfrost-icebreaker.svg" alt="Blockfrost Icebreaker logo" className="size-15.5" />
        <img src="/images/workloads-gray/quantum-hosky.svg" alt="Quantum Hosky logo" className="size-15.5" />
        <img src="/images/workloads-gray/deltadefi.png" alt="Deltadefi logo" className="size-15.5" />
        <img src="/images/workloads-gray/fluidtokens.png" alt="Fluidtokens logo" className="size-15.5" />
      </div>

      <div className="grid auto-rows-min gap-6 whitespace-pre-wrap">
        <h2 className="text-3xl/[40px] md:text-4xl font-semibold text-zinc-800">
          {'Curated catalog of\nblockchain workloads'}
        </h2>

        <p className="text-zinc-500 max-w-[596px]">
          A curated collection of blockchain workloads, refined through close collaboration with ecosystem teams.
          SuperNode manages setup, integrations, and dependencies, providing operators with a smoother path to
          production.
        </p>

        <SearchInput
          onSearchText={handleSearchText}
          className="max-w-none"
          inputClassName="text-[#FF007F] placeholder:text-[#FF007F]"
          disableShortcuts
        />
      </div>
    </Section>
  );
}
