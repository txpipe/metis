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
      normal: 'py-3 px-8 text-lg leading-none',
    },
    color: {
      zinc: '',
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
    radius: 'full',
    fullWidth: false,
    disabled: false,
    loading: false,
  },

  compoundVariants: [
    { variant: 'solid', color: 'zinc', class: 'bg-zinc-800 text-white' },
    { variant: 'outlined', color: 'zinc', class: 'border-zinc-800 text-zinc-800' },
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
