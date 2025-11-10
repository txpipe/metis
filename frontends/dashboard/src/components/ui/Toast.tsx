import { tv, type VariantProps } from 'tailwind-variants';
import { toast as hotToast } from 'react-hot-toast';

// Components
import { ToastSuccessIcon } from '~/components/icons/ToastSuccessIcon';
import { XIcon } from '~/components/icons/XIcon';

const toast = tv({
  base: 'border rounded-lg p-5 shadow-[0px_4px_16px_0px_rgba(16,11,39,0.08)] flex flex-row items-start gap-4',
  variants: {
    style: {
      success: 'border-[#48C1B5] bg-[#F6FFF9]',
    },
  },

  defaultVariants: {
    style: 'success',
  },
});

export type Props = VariantProps<typeof toast> & {
  toastId: string;
  className?: string;
  title?: string;
  message?: string;
};

export function Toast({ title, message, toastId, ...variantProps }: Props) {
  return (
    <div className={toast(variantProps)}>
      {variantProps.style === 'success' && (
        <ToastSuccessIcon />
      )}
      <div className="flex flex-col gap-1">
        {title && <div className="text-[#27303A] font-semibold text-sm">{title}</div>}
        {message && <div className="text-[#2F3F53] text-xs">{message}</div>}
      </div>
      <button
        type="button"
        onClick={() => {
          hotToast.dismiss(toastId);
        }}
        className="cursor-pointer"
      >
        <XIcon className="w-4.5 h-4.5 text-[#979FA9]" />
      </button>
    </div>
  );
}
