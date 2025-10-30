import type { PropsWithChildren } from 'react';
import { tv, type VariantProps } from 'tailwind-variants';

// Components
import { AlertCircleIcon } from '~/components/icons/AlertCircleIcon';

const callout = tv({
  slots: {
    base: 'p-6 border rounded-sm flex gap-3 items-center',
    icon: '',
  },
  variants: {
    type: {
      warning: {
        base: 'border-[#F29B06]/[96.86%] bg-[#F29B06]/2 text-[#2B2B2B]',
        icon: 'text-[#F29B06]',
      },
      note: {
        base: 'border-[#0000FF]/40 bg-[#0000FF]/2 text-[#2B2B2B]',
        icon: 'text-[#0000FF]',
      },
    },
  },
});

type CalloutProps = VariantProps<typeof callout> & {
  className?: string;
};

export function Callout({ children, className, ...variantProps }: PropsWithChildren<CalloutProps>) {
  const { base, icon } = callout(variantProps);
  return (
    <div className={base({ className })}>
      <AlertCircleIcon className={icon()} />
      {children}
    </div>
  );
}
