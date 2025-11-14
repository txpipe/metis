import { twMerge } from 'tailwind-merge';

interface CommonProps {
  className?: string;
}

export function Card({ children, className }: React.PropsWithChildren<CommonProps>) {
  return (
    <div
      className={twMerge(
        'bg-[#F9F9F9] border-[0.5px] border-[#CBD5E1] rounded-xl p-6 flex flex-col',
        className,
      )}
    >

      {children}
    </div>
  );
}

export function CardTitle({ children, className }: React.PropsWithChildren<CommonProps>) {
  return (
    <h2 className={twMerge('text-[22px]/[22px] font-semibold text-[#686868]', className)}>
      {children}
    </h2>
  );
}
