import type { SVGProps } from 'react';

// Tabler Icon => alert-circle
export function AlertCircleIcon({ strokeWidth = 2, ...props }: SVGProps<SVGSVGElement>) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="24"
      height="24"
      fill="currentColor"
      strokeWidth={strokeWidth}
      viewBox="0 0 24 24"
      {...props}
    >
      <path fill="none" d="M0 0h24v24H0z" />
      <path d="M12 2a10 10 0 0 1 10 10 10 10 0 0 1-20 .32v-.6A10 10 0 0 1 12 2m.01 13h-.13a1 1 0 0 0 0 2h.25a1 1 0 0 0 0-2zM12 7a1 1 0 0 0-1 .88v4.24a1 1 0 0 0 2 0V7.88A1 1 0 0 0 12 7" />
    </svg>
  );
}
