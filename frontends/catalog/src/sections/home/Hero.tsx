import { Link } from '@tanstack/react-router';

// Components
import { StarIcon } from '~/components/icons';
import { button } from '~/components/ui/Button';

export function HeroSection() {
  const hlClass = 'font-bold text-zinc-700';

  return (
    <div className="bg-[url('/images/home-bg.svg')] flex items-center justify-center bg-center min-h-[calc(100dvh-145px)]">
      <div className="text-center w-full max-w-245">
        <div className="flex flex-col items-center gap-14 w-full">
          <h1 className="font-bold text-4xl/[48px] md:text-7xl/[89px] text-zinc-800 text-center">
            Supercharge your<br />blockchain infrastructure
          </h1>

          <p className="text-md md:text-lg text-zinc-500 font-medium">
            A unified <span className={hlClass}>open-source</span> platform to deploy and manage all your&nbsp;
            <span className={hlClass}>blockchain</span> infrastructure without compromising&nbsp;
            <span className={hlClass}>decentralization</span>. Bring your <span className={hlClass}>own cloud</span>,
            choose what to run, and start earning <span className={hlClass}>rewards</span>.
          </p>

          <div className="flex gap-6.75 flex-wrap justify-center">
            <a
              href="#beta"
              className={button({ variant: 'outlined', color: 'zinc', size: 'normal' })}
            >
              Register to beta
            </a>
            <Link to="/catalog" className={button({ variant: 'solid', color: 'zinc', size: 'normal', className: 'gap-2.5' })}>
              Explore workloads <StarIcon className="size-4" />
            </Link>
          </div>
        </div>
      </div>
    </div>
  );
}
