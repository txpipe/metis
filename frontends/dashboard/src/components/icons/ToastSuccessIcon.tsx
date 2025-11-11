import type { SVGProps } from 'react';

// From figma
export function ToastSuccessIcon(props: SVGProps<SVGSVGElement>) {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="none" viewBox="0 0 24 24" {...props}>
      <rect width="24" height="24" fill="url(#success_a)" rx="6" />
      <path stroke="#fff" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.5" d="m8.5 12.5 2 2 5-5" />
      <defs>
        <linearGradient id="success_a" x1="12" x2="12" y1="0" y2="24" gradientUnits="userSpaceOnUse">
          <stop stopColor="#48ca93" />
          <stop offset="1" stopColor="#48baca" />
        </linearGradient>
      </defs>
    </svg>
  );
}
