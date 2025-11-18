import { Section } from '~/components/Section';

export function CuratedCatalogSection() {
  return (
    <Section
      title={'Curated catalog of\nblockchain workloads'}
      description="A curated collection of blockchain workloads, refined through close collaboration with ecosystem teams. SuperNode manages setup, integrations, and dependencies, providing operators with a smoother path to production."
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
        <img src="/images/workloads-gray/charli3.png" alt="Charli 3 logo" className="size-15.5" />
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
    </Section>
  );
}
