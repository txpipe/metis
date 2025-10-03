import type { SVGProps } from 'react';

// Icon from Figma
export function LinkIcon({ strokeWidth = 2, ...props }: SVGProps<SVGSVGElement>) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={strokeWidth}
      viewBox="0 0 16 16"
      {...props}
    >
      <path d="M7.33 5.17H4A1.33 1.33 0 0 0 2.67 6.5v6A1.33 1.33 0 0 0 4 13.83h6a1.33 1.33 0 0 0 1.33-1.33V9.17m-4.66.66 6.66-6.66m0 0H10m3.33 0V6.5" />
    </svg>
  );
}
