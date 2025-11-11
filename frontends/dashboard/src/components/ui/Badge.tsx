import { tv, type VariantProps } from 'tailwind-variants';

const badge = tv({
  base: 'rounded-full text-xs w-fit',
  variants: {
    style: {
      default: 'font-semibold border border-black',
      status: 'text-[#0000FF] bg-[#0000FF]/8',
      error: 'text-[#FF7474] bg-[#FF7474]/8',
      success: 'text-[#69C876] bg-[#69C876]/8',
      pause: 'text-[#2B2B2B] bg-black/8',
    },
    size: {
      default: 'px-2.75 py-0.75',
      small: 'px-1.5 py-[1.5px]',
    },
  },

  defaultVariants: {
    style: 'default',
    size: 'default',
  },
});

export type Props = VariantProps<typeof badge> & {
  className?: string;
  label: string;
};

export function Badge({ label, ...variantProps }: Props) {
  return (
    <div className={badge(variantProps)}>
      {label}
    </div>
  );
}
