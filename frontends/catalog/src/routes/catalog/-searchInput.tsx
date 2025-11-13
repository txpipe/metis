import { useRef } from 'react';
import { useHotkeys } from 'react-hotkeys-hook';

// Components
import { SearchIcon } from '~/components/icons';

interface Props {
  onSearchText: (text: string | null) => void;
}

export function SearchInput({ onSearchText }: Props) {
  const inputRef = useRef<HTMLInputElement>(null);

  useHotkeys('mod+k', e => {
    e.preventDefault();
    inputRef.current?.focus();
  });

  return (
    <div className="relative max-w-150 w-full">
      <SearchIcon className="absolute text-zinc-400 pointer-events-none top-1/2 -translate-y-1/2 left-3" />
      <input
        ref={inputRef}
        onChange={e => onSearchText(e.currentTarget.value || null)}
        type="text"
        name="search"
        className="w-full pl-10 in-[.os-macos]:pr-14 not-in-[.os-macos]:pr-18 text-zinc-700 placeholder:text-zinc-400 border-none ring-zinc-400 rounded-md shadow-[0px_2px_5px_0px_rgba(32,42,54,0.06),0px_1px_5px_-4px_rgba(19,19,22,0.4),0px_0px_0px_1px_rgba(33,33,38,0.1)]"
        placeholder="Search"
      />
      <div className="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 text-xs text-zinc-600 bg-white py-0.5 px-1.5 rounded-sm shadow-[0px_0px_0px_1px_rgba(238,238,240,1)_inset,0px_0px_0px_0px_rgba(255,255,255,1)_inset]">
        <kbd className="hidden in-[.os-macos]:block">⌘ K</kbd>
        <kbd className="hidden not-in-[.os-macos]:block">Ctrl K</kbd>
      </div>
    </div>
  );
}
