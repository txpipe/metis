import type { SVGProps } from 'react';

// Custom Icon
export function SpinnerIcon(props: SVGProps<SVGSVGElement>) {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" fill="none" viewBox="0 0 18 18" {...props}>
      <g clipPath="url(#a)">
        <foreignObject width="2074.32" height="2074.32" x="-1037.16" y="-1037.16" transform="translate(9 9)scale(.009)">
          <div
            // @ts-ignore => Exported from figma
            xmlns="http://www.w3.org/1999/xhtml"
            style={{
              background: 'conic-gradient(from 90deg,rgba(255,255,255,.698) 0deg,rgba(255,255,255,0) .036deg,#fff 360deg)',
              height: '100%',
              width: '100%',
              opacity: 1,
            }}
          />
        </foreignObject>
      </g>
      <path d="M9 18A9 9 0 1 0 9 0a9 9 0 0 0 0 18m0-1.5a7.5 7.5 0 1 0 0-15 7.5 7.5 0 0 0 0 15" clipRule="evenodd" />
      <defs>
        <clipPath id="a">
          <path fillRule="evenodd" d="M9 18A9 9 0 1 0 9 0a9 9 0 0 0 0 18m0-1.5a7.5 7.5 0 1 0 0-15 7.5 7.5 0 0 0 0 15" clipRule="evenodd" />
        </clipPath>
      </defs>
    </svg>
  );
}
