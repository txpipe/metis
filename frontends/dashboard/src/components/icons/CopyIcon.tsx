import type { SVGProps } from 'react';

// Tabler Icon => Copy
export function CopyIcon({ strokeWidth = 2, ...props }: SVGProps<SVGSVGElement>) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="24"
      height="24"
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={strokeWidth}
      viewBox="0 0 24 24"
      {...props}
    >
      <path stroke="none" d="M0 0h24v24H0z" />
      <path d="M7 9.67A2.67 2.67 0 0 1 9.67 7h8.66A2.67 2.67 0 0 1 21 9.67v8.66A2.67 2.67 0 0 1 18.33 21H9.67A2.67 2.67 0 0 1 7 18.33z" />
      <path d="M4.01 16.74A2 2 0 0 1 3 15V5c0-1.1.9-2 2-2h10c.75 0 1.16.38 1.5 1" />
    </svg>
  );
}
