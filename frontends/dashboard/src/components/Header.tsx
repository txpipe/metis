import { Link } from '@tanstack/react-router';

// Icons
import { LogoIcon } from '~/components/icons/LogoIcon';

interface Props {}

export function Header({}: Props) {
  return (
    <header className="w-full bg-white z-1 border-b border-neutral-200 grid grid-cols-1 md:grid-cols-[348px_auto_1fr_auto] lg:grid-cols-[348px_auto_1fr_348px] items-center gap-2 md:gap-0 pb-2 md:pb-0">
      {/* Logo */}
      <div className="flex items-center justify-between md:justify-center gap-3 py-4 md:py-0 px-8 md:px-0">
        <Link to="/" className="w-fit flex text-2xl items-center gap-1.5 font-poppins text-zinc-900">
          <LogoIcon className="h-8.25" />
          <span>
            SUPER<span className="font-bold">NODE</span>
          </span>
        </Link>
      </div>
      {/* Separator */}
      <div className="h-23.5 w-px bg-zinc-200 hidden md:block" />
      <nav className="px-6 sm:px-12 flex flex-row gap-8 font-medium text-zinc-800">
        <Link to="/" data-active>Dashboard</Link>
      </nav>
    </header>
  );
}
