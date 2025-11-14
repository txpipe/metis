export function GreenCheckIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width="24"
      height="24"
      fill="none"
      viewBox="0 0 24 24"
      {...props}
    >
      <circle cx="12" cy="12" r="12" fill="#69c876" transform="rotate(90 12 12)" />
      <path fill="#fff" fillRule="evenodd" d="M16.8 8.77c.3.3.3.8 0 1.1l-6.28 6.28c-.3.31-.8.31-1.1 0l-3.14-3.13a.78.78 0 1 1 1.1-1.11l2.59 2.58 5.72-5.72c.3-.3.8-.3 1.11 0" clipRule="evenodd" />
    </svg>
  );
}
