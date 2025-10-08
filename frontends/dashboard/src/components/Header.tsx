import { Link } from '@tanstack/react-router';

interface Props {}

export function Header({}: Props) {
  return (
    <header className="w-full border-b border-neutral-200 grid grid-cols-[350px_2px_1fr] items-center">
      {/* Logo */}
      <div className="flex items-center justify-center gap-3">
        <Link to="/" className="w-fit flex text-4xl font-bold items-center gap-1.5">
          <img src="/logo.svg" alt="Metis Logo" className="h-[44.27px]" />
          Metis
        </Link>
        <div className="text-[#2B2B2B]/30 mt-2.5">
          By Txpipe
        </div>
      </div>
      {/* Separator */}
      <div className="h-23.75 bg-[#F4F4F4]" />
      {/* Navbar */}
      <nav className="px-12 flex flex-row gap-8">
        <Link to="/" data-active>Dashboard</Link>
      </nav>
    </header>
  );
}
