import type { Route } from './+types/_index';

export function meta({}: Route.MetaArgs) {
  return [
    { title: 'Metis' },
    { name: 'description', content: 'Welcome to Metis!' },
  ];
}

export default function Home({}: Route.ComponentProps) {
  return (
    <></>
  );
}
