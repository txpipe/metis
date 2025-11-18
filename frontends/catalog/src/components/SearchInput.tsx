import { useEffect, useRef, useState } from 'react';
import { useHotkeys } from 'react-hotkeys-hook';
import { useDebounce } from '@uidotdev/usehooks';

// Components
import { SearchIcon } from '~/components/icons';

interface Props {
  onSearchText: (text: string | null) => void;
  debounceMs?: number;
}

export function SearchInput({ onSearchText, debounceMs = 300 }: Props) {
  const [val, setVal] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const debouncedVal = useDebounce<string | null>(val, debounceMs);

  useHotkeys('mod+k', e => {
    e.preventDefault();
    inputRef.current?.focus();
  });

  useEffect(() => {
    if (debouncedVal !== null) {
      onSearchText(debouncedVal);
    }
  }, [debouncedVal, onSearchText]);

  return (
    <div className="relative max-w-92.5 w-full">
      <SearchIcon className="absolute text-zinc-400 pointer-events-none top-1/2 -translate-y-1/2 left-3" />
      <input
        ref={inputRef}
        onChange={e => setVal(e.currentTarget.value)}
        type="text"
        name="search"
        className="w-full pl-10 in-[.os-macos]:pr-14 not-in-[.os-macos]:pr-18 text-zinc-700 placeholder:text-zinc-400 border border-zinc-200 ring-zinc-400 rounded-md"
        placeholder="Search workloads"
      />
      <div className="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 text-xs text-zinc-600 bg-white py-0.5 px-1.5 rounded-sm border border-zinc-300">
        <kbd className="hidden in-[.os-macos]:block">⌘ K</kbd>
        <kbd className="hidden not-in-[.os-macos]:block">Ctrl K</kbd>
      </div>
    </div>
  );
}
