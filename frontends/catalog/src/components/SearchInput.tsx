import { useEffect, useRef, useState } from 'react';
import { useHotkeys } from 'react-hotkeys-hook';
import { useDebounce } from '@uidotdev/usehooks';
import { twMerge } from 'tailwind-merge';

// Components
import { SearchIcon } from '~/components/icons';

interface Props {
  onSearchText: (text: string | null) => void;
  disableShortcuts?: boolean;
  debounceMs?: number;
  className?: string;
  inputClassName?: string;
}

export function SearchInput({
  onSearchText,
  className,
  inputClassName,
  debounceMs = 300,
  disableShortcuts = false,
}: Props) {
  const [val, setVal] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const debouncedVal = useDebounce<string | null>(val, debounceMs);

  useHotkeys('mod+k', e => {
    if (disableShortcuts) return;
    e.preventDefault();
    inputRef.current?.focus();
  });

  useEffect(() => {
    if (debouncedVal !== null) {
      onSearchText(debouncedVal);
    }
  }, [debouncedVal, onSearchText]);

  return (
    <div className={twMerge('relative max-w-92.5 w-full', className)}>
      <SearchIcon className={twMerge('absolute text-zinc-400 pointer-events-none top-1/2 -translate-y-1/2 left-3', inputClassName)} />
      <input
        ref={inputRef}
        onChange={e => setVal(e.currentTarget.value)}
        type="text"
        name="search"
        className={twMerge('w-full pl-10 in-[.os-macos]:pr-14 not-in-[.os-macos]:pr-18 text-zinc-700 placeholder:text-zinc-400 border border-zinc-200 ring-zinc-400 rounded-md', inputClassName)}
        placeholder="Search workloads"
      />
      {!disableShortcuts && (
        <div className="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 text-xs text-zinc-600 bg-white py-0.5 px-1.5 rounded-sm border border-zinc-300">
          <kbd className="hidden in-[.os-macos]:block">⌘ K</kbd>
          <kbd className="hidden not-in-[.os-macos]:block">Ctrl K</kbd>
        </div>
      )}
    </div>
  );
}
