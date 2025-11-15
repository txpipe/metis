import { twMerge } from 'tailwind-merge';
import { clsx } from 'clsx';

interface Props {
  id?: string;
  title?: string;
  description?: string;
  center?: boolean;
  sideBySide?: boolean;
  className?: string;
}

export function Section({
  id, title, description, center, sideBySide, className, children,
}: React.PropsWithChildren<Props>) {
  return (
    <section
      id={id}
      className={clsx('w-full', className)}
    >
      <div
        className={twMerge(
          'py-8 md:py-16 px-6 md:px-12 lg:px-37.5 mx-auto max-w-[1440px] grid gap-6 md:gap-12',
          sideBySide ? 'auto-cols-fr md:grid-flow-col' : 'auto-rows-auto',
        )}
      >
        {(title || description) && (
          <div className="grid auto-rows-min gap-6 whitespace-pre-wrap">
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
      </div>
    </section>
  );
}
