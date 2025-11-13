export function StarIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="16"
      height="16"
      fill="none"
      viewBox="0 0 16 16"
      {...props}
    >
      <path
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M3 14.67v-3.34m0-6.66V1.33M1.33 3h3.34M1.33 13h3.34m4-11L7.5 5c-.19.5-.28.74-.43.94a2 2 0 0 1-.47.48c-.2.14-.45.24-.94.42L2.67 8l3 1.16c.5.18.74.28.94.42a2 2 0 0 1 .47.48c.15.2.24.45.43.93L8.67 14l1.15-3c.2-.5.29-.74.43-.94a2 2 0 0 1 .47-.48c.2-.14.45-.24.94-.42l3-1.16-3-1.16c-.49-.18-.73-.28-.94-.42a2 2 0 0 1-.47-.48c-.14-.2-.24-.45-.43-.93z"
      />
    </svg>
  );
}
