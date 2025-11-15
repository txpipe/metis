import type { MouseEventHandler, PropsWithChildren } from 'react';
import { tv, type VariantProps } from 'tailwind-variants';

export const button = tv({
  base: [
    'group flex items-center justify-center cursor-pointer truncate',
  ],

  variants: {
    variant: {
      solid: '',
      outlined: 'border',
    },
    size: {
      small: 'py-0.5 px-6 text-xs font-medium',
      'small-auto-x': 'py-0.5 text-xs font-medium',
      normal: 'py-2.5 px-8',
    },
    text: {
      xs: 'text-xs/none',
      sm: 'text-sm/none',
      base: 'text-base/none',
      lg: 'text-lg/none',
    },
    color: {
      zinc: '',
      blue: '',
      red: '',
      green: '',
    },
    radius: {
      none: 'rounded-none',
      sm: 'rounded-sm',
      md: 'rounded-md',
      lg: 'rounded-lg',
      full: 'rounded-full',
    },
    fullWidth: {
      true: 'w-full',
    },
    disabled: {
      true: 'opacity-24 pointer-events-none cursor-not-allowed',
    },
    loading: {
      true: 'pointer-events-none',
    },
  },

  defaultVariants: {
    size: 'normal',
    variant: 'solid',
    color: 'zinc',
    text: 'lg',
    radius: 'full',
    fullWidth: false,
    disabled: false,
    loading: false,
  },

  compoundVariants: [
    { variant: 'solid', color: 'zinc', class: 'bg-zinc-800 text-white' },
    { variant: 'outlined', color: 'zinc', class: 'border-zinc-800 text-zinc-800' },

    { variant: 'solid', color: 'blue', class: 'bg-[#F8F8FF]/50 text-[#0000FF] border-[0.5px] border-[#0600FF]/50' },
    { variant: 'outlined', color: 'blue', class: 'border-[#0600FF] text-[#0000FF]' },

    { variant: 'solid', color: 'green', class: 'bg-[#69C876]/8 text-[#69C876] border-[0.5px] border-[#69C876]/50' },
    { variant: 'outlined', color: 'green', class: 'border-[#69C876] text-[#69C876]' },

    { variant: 'solid', color: 'red', class: 'bg-[#FF7474]/8 text-[#FF7474] border-[0.5px] border-[#FF7474]/50' },
    { variant: 'outlined', color: 'red', class: 'border-[#FF7474] text-[#FF7474]' },
  ],
});

type ButtonProps = VariantProps<typeof button> & {
  type?: 'button' | 'submit' | 'reset';
  className?: string;
  onClick?: MouseEventHandler<HTMLButtonElement>;
};

export function Button({ type, children, onClick, ...props }: PropsWithChildren<ButtonProps>) {
  const className = button(props);

  return (
    <button type={type} onClick={onClick} className={className} disabled={props.disabled}>
      {children}
    </button>
  );
}
