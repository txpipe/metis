import clsx from 'clsx';

interface Props {
  title?: string;
  titleAction?: React.ReactNode;
  className?: string;
}

export function Card({ title, children, className, titleAction }: React.PropsWithChildren<Props>) {
  return (
    <div
      className={clsx(
        'bg-[#F9F9F9] border-[0.5px] border-[#CBD5E1] rounded-xl p-6 grid grid-rows-[auto_1fr]',
        className,
      )}
    >
      <div className="flex flex-row justify-between items-center">
        <h2 className="text-[22px]/[22px] font-semibold text-[#686868]">{title}</h2>
        {titleAction}
      </div>

      {children}
    </div>
  );
}
