import { clsx } from 'clsx';
interface Props {
  className?: string;
}

export function Card({ children, className }: React.PropsWithChildren<Props>) {
  return (
    <div className={clsx('flex flex-col gap-8 bg-zinc-50 p-8 rounded-xl text-zinc-800', className)}>
      {children}
    </div>
  );
}

export function CardHeader({ children }: React.PropsWithChildren<Props>) {
  return (
    <div className="text-2xl font-semibold">
      {children}
    </div>
  );
}

export function CardBody({ children, className }: React.PropsWithChildren<Props>) {
  return (
    <div className={className}>
      {children}
    </div>
  );
}
