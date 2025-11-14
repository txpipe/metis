import { twMerge } from 'tailwind-merge';
import { clsx } from 'clsx';

interface Props {
  id?: string;
  title?: string;
  description?: string;
  center?: boolean;
  className?: string;
}

export function Section({ id, title, description, center, className, children }: React.PropsWithChildren<Props>) {
  return (
    <section id={id} className={twMerge('py-16 px-37.5 w-full grid auto-rows-auto gap-12', className)}>
      {(title || description) && (
        <div className="grid auto-rows-min gap-6">
          {title && (
            <h2 className={clsx('text-3xl/[40px] md:text-4xl font-semibold text-zinc-800', center && 'mx-auto text-center')}>
              {title}
            </h2>
          )}
          {description && (
            <p className={clsx('text-zinc-500 max-w-[596px]', center && 'mx-auto text-center')}>
              {description}
            </p>
          )}
        </div>
      )}

      {children}
    </section>
  );
}
