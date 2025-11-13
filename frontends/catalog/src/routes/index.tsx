import { createFileRoute, Link } from '@tanstack/react-router';

// Components
import { CategoryRow } from '~/components/CategoryRow';
import { Container } from '~/components/ui/Container';
import { button } from '~/components/ui/Button';
import { StarIcon } from '~/components/icons';

export const Route = createFileRoute('/')({
  component: LandingPage,
});

function LandingPage() {
  const hlClass = 'font-bold text-zinc-700';
  return (
    <Container className="bg-[url('/images/home-bg.svg')] flex items-center justify-center bg-center">
      <div className="text-center w-full max-w-245">
        <div className="flex flex-col items-center gap-14 w-full">
          <h1 className="font-bold text-4xl/[48px] md:text-7xl/[89px] text-zinc-800 text-center">
            A single tool to run all your UTxO workloads
          </h1>

          <p className="text-md md:text-lg leading- text-zinc-500 font-medium">
            An <span className={hlClass}>open-source</span> tool built for blockchain&nbsp;
            <span className={hlClass}>operators</span>. Deploy and manage multiple nodes and&nbsp;
            <span className={hlClass}>workloads</span> in a few clicks—no custom setups, no hidden dependencies.
            Modular by design: bring your <span className={hlClass}>own cloud</span> and choose what to run.
            Don't miss out on <span className={hlClass}>rewards</span>.
          </p>

          <div className="flex gap-6.75 flex-wrap justify-center">
            <a
              href="https://github.com/txpipe/metis/blob/main/README.md"
              target="_blank"
              rel="noopener noreferrer"
              className={button({ variant: 'outlined', color: 'zinc', size: 'normal' })}
            >
              Read the docs
            </a>
            <Link to="/catalog" className={button({ variant: 'solid', color: 'zinc', size: 'normal', className: 'gap-2.5' })}>
              Explore Catalog <StarIcon className="size-4" />
            </Link>
          </div>

          <CategoryRow className="gap-4" />
        </div>
      </div>
    </Container>
  );
}
