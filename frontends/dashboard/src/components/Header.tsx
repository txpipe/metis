import { Link } from '@tanstack/react-router';

interface Props {}

export function Header({}: Props) {
  return (
    <header className="w-full bg-white z-1 border-b border-neutral-200 grid grid-cols-1 sm:grid-cols-[348px_1px_1fr] items-center gap-4 sm:gap-0">
      {/* Logo */}
      <div className="flex items-center justify-center gap-3 py-4 sm:py-0">
        <Link to="/" className="w-fit flex text-2xl items-center gap-1.5 font-poppins text-zinc-900">
          <img src="/logo.svg" alt="SuperNode Logo" className="h-9.5" />
          <span>
            SUPER<span className="font-bold">NODE</span>
          </span>
        </Link>
      </div>
      {/* Separator */}
      <div className="h-23.5 bg-zinc-200 hidden sm:block" />
      {/* Navbar */}
      <nav className="px-6 sm:px-12 flex flex-row gap-8 font-medium text-zinc-800">
        <Link to="/" data-active>Dashboard</Link>
      </nav>
    </header>
  );
}
