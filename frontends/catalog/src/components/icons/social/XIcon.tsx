export function XIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" viewBox="0 0 24 24" {...props}>
      <g mask="url(#social_x_a)">
        <path d="M17.96 3.95h2.61l-5.7 6.53 6.7 8.88h-5.24l-4.11-5.39-4.7 5.39h-2.6L11 12.38 4.58 3.95h5.38l3.7 4.92zm-.91 13.84h1.44L9.17 5.43H7.62z" />
      </g>
      <mask id="social_x_a" width="18" height="18" x="4" y="3" maskUnits="userSpaceOnUse" style={{ maskType: 'luminance' }}>
        <path fill="#fff" d="M4.58 3.15h17v17h-17z" />
      </mask>
    </svg>
  );
}
