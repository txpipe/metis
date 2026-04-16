import { type KeyboardEvent, type ReactNode, useId, useRef, useState } from 'react';
import clsx from 'clsx';

import { InfoCircleIcon } from '~/components/icons/InfoCircleIcon';

type Props = {
  content: ReactNode;
  className?: string;
};

export function InfoTooltip({ content, className }: Props) {
  const [open, setOpen] = useState(false);
  const tooltipId = useId();
  const buttonRef = useRef<HTMLButtonElement>(null);

  const handleKeyDown = (event: KeyboardEvent<HTMLButtonElement>) => {
    if (event.key !== 'Escape') {
      return;
    }

    setOpen(false);
    buttonRef.current?.blur();
  };

  return (
    <span
      className={clsx('relative inline-flex items-center', className)}
      onMouseEnter={() => setOpen(true)}
      onMouseLeave={() => setOpen(false)}
    >
      <button
        ref={buttonRef}
        type="button"
        className="inline-flex cursor-help items-center text-[#969FAB] outline-none transition-colors hover:text-[#64748B] focus-visible:text-[#64748B]"
        aria-label="Show metric description"
        aria-describedby={open ? tooltipId : undefined}
        onFocus={() => setOpen(true)}
        onBlur={() => setOpen(false)}
        onClick={() => setOpen(prev => !prev)}
        onKeyDown={handleKeyDown}
      >
        <InfoCircleIcon className="h-3 w-3" />
      </button>

      {open && (
        <span
          id={tooltipId}
          role="tooltip"
          className="absolute bottom-[calc(100%+8px)] left-1/2 z-20 w-56 -translate-x-1/2 rounded-lg border border-zinc-200 bg-white px-3 py-2 text-left text-xs font-medium leading-5 text-[#2B2B2B]/80 shadow-lg"
        >
          {content}
        </span>
      )}
    </span>
  );
}
