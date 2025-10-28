import type { SVGProps } from 'react';

// Tabler Icon: circle-check
export function CircleCheckIcon(props: SVGProps<SVGSVGElement>) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="24"
      height="24"
      fill="currentColor"
      viewBox="0 0 24 24"
      {...props}
    >
      <path fill="none" d="M0 0h24v24H0z" />
      <path d="M17 3.34a10 10 0 1 1-15 8.98v-.64a10 10 0 0 1 15-8.34m-1.3 5.95a1 1 0 0 0-1.31-.08l-.1.08L11 12.6l-1.3-1.3-.09-.08a1 1 0 0 0-1.4 1.4l.08.1 2 2 .1.08a1 1 0 0 0 1.22 0l.1-.08 4-4 .08-.1a1 1 0 0 0-.08-1.32" />
    </svg>
  );
}
