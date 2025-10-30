import type { SVGProps } from 'react';

// Icon from figma
export function CaretRightIcon(props: SVGProps<SVGSVGElement>) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      fill="none"
      stroke="currentColor"
      viewBox="0 0 16 16"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth="1.33"
      {...props}
    >
      <path d="m6 12 4-4-4-4" />
    </svg>
  );
}
