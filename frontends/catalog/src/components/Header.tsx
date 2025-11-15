import { Link, useLocation, useNavigate } from '@tanstack/react-router';
import { useCallback } from 'react';

// Icons
import { DiscordIcon, GithubIcon, XIcon } from '~/components/icons/social';

// Catalog route
import { Route as CatalogRoute } from '~/routes/catalog';

// Components
import { SearchInput } from '~/components/SearchInput';
import { LogoIcon } from '~/components/icons/LogoIcon';

interface Props {}

export function Header({}: Props) {
  const navigate = useNavigate({ from: CatalogRoute.fullPath });
  const location = useLocation();

  const handleSearchText = useCallback((text: string | null) => {
    navigate({ to: '/catalog', search: prev => ({ ...prev, query: !!text ? text : undefined }), replace: true });
  }, [navigate]);
  return (
    <header className="w-full bg-white z-1 border-b border-neutral-200 grid grid-cols-1 md:grid-cols-[348px_auto_1fr_auto] lg:grid-cols-[348px_auto_1fr_348px] items-center gap-4 md:gap-0 pb-2 md:pb-0">
      {/* Logo */}
      <div className="flex items-center justify-center gap-3 py-4 md:py-0">
        <Link to="/" className="w-fit flex text-2xl items-center gap-1.5 font-poppins text-zinc-900">
          <LogoIcon className="h-9.5 text-[#4A4A4A]" />
          <span>
            SUPER<span className="font-bold">NODE</span>
          </span>
        </Link>
      </div>
      {/* Separator */}
      <div className="h-23.5 bg-zinc-200 hidden md:block" />
      {/* Search Bar (centered) */}
      <div className="flex flex-col items-center justify-center px-6 gap-2">
        <div className="relative w-full max-w-92.5">
          <SearchInput onSearchText={handleSearchText} />
          {location.pathname !== '/catalog' && (
            <div className="absolute -right-5 translate-x-full top-0 bottom-0 items-center gap-5 hidden lg:flex">
              <div className="w-px h-full bg-zinc-200" />
              <Link to="/catalog" className="font-medium text-zinc-800">Explore workloads</Link>
            </div>
          )}
        </div>
        {location.pathname !== '/catalog' && (
          <Link to="/catalog" className="text-sm font-medium text-zinc-800 block lg:hidden">Explore workloads</Link>
        )}
      </div>

      {/* Right Section */}
      <div className="flex items-center justify-center md:justify-end gap-4 px-12 text-zinc-800">
        <a href="https://discord.gg/eVc6HJrYmP" target="_blank" rel="noopener noreferrer" className="hover:text-zinc-600">
          <DiscordIcon />
        </a>
        <a href="https://x.com/txpipe_tools" target="_blank" rel="noopener noreferrer" className="hover:text-zinc-600">
          <XIcon />
        </a>
        <a href="https://github.com/txpipe/metis" target="_blank" rel="noopener noreferrer" className="hover:text-zinc-600">
          <GithubIcon />
        </a>
      </div>
    </header>
  );
}
