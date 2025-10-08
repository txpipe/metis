import type { MouseEventHandler, PropsWithChildren } from 'react';
import { tv, type VariantProps } from 'tailwind-variants';

const button = tv({
  base: [
    'group flex items-center justify-center cursor-pointer truncate',
    'py-1.25 px-8',
  ],

  variants: {
    variant: {
      solid: '',
      outlined: 'border bg-transparent',
    },
    color: {
      default: '',
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
    size: 'md',
    variant: 'solid',
    color: 'default',
    radius: 'md',
    fullWidth: false,
    disabled: false,
    loading: false,
  },

  compoundVariants: [
    { variant: 'solid', color: 'default', class: 'bg-[#2B2B2B] text-white' },
    { variant: 'outlined', color: 'default', class: 'border-[#2B2B2B] text-[#2B2B2B]' },
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
    <button type={type} onClick={onClick} className={className}>
      {children}
    </button>
  );
}
