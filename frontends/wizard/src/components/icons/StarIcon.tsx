import type { SVGProps } from 'react';

// Tabler Icon => star
export function StarIcon({ strokeWidth = 2, filled, ...props }: SVGProps<SVGSVGElement> & { filled?: boolean; }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="24"
      height="24"
      fill={filled ? 'currentColor' : 'none'}
      stroke={filled ? 'none' : 'currentColor'}
      strokeWidth={strokeWidth}
      viewBox="0 0 24 24"
      {...props}
    >
      {!filled
        ? (
          <>
            <path stroke="none" d="M0 0h24v24H0z" />
            <path d="M12 17.75 5.83 21 7 14.12 2 9.26l6.9-1L11.99 2l3.09 6.25 6.9 1-5 4.87L18.16 21z" />
          </>
        )
        : (
          <>
            <path fill="none" d="M0 0h24v24H0z" />
            <path d="m8.24 7.34-6.38.93-.11.02a1 1 0 0 0-.44 1.68l4.62 4.5-1.09 6.36-.01.1a1 1 0 0 0 1.46.95l5.7-3 5.7 3 .1.05a1 1 0 0 0 1.35-1.1l-1.09-6.36 4.63-4.5.07-.08a1 1 0 0 0-.63-1.62l-6.38-.93-2.85-5.78a1 1 0 0 0-1.8 0z" />
          </>
        )}
    </svg>
  );
}
