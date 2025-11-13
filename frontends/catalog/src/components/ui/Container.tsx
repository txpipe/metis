import { twMerge } from 'tailwind-merge';

interface Props {
  className?: string;
}

export function Container({ children, className }: React.PropsWithChildren<Props>) {
  return (
    <main className={twMerge('px-4 sm:px-14 py-8 max-w-dvw', className)}>
      {children}
    </main>
  );
}
