import { Menu, MenuButton, MenuItem, MenuItems } from '@headlessui/react';
import clsx from 'clsx';

// Components
import { Button } from '~/components/ui/Button';
import { SpinnerIcon } from '~/components/icons/SpinnerIcon';

// Context
import { useWallet } from '~/contexts/wallet';

interface Props {
  variant: 'solid' | 'outlined';
  fullWidth?: boolean;
  className?: string;
}

export function ConnectWallet({ variant, fullWidth, className }: Props) {
  const { isConnecting, availableWallets, connectWallet, connectedWalletDetails } = useWallet();

  return (
    <Menu as="div" className={clsx(className, fullWidth ? 'w-full max-w-full' : 'w-fit max-w-full')}>
      <MenuButton disabled={isConnecting || !!connectedWalletDetails} as="div">
        {connectedWalletDetails
          ? (
            <Button type="button" variant={variant} fullWidth={fullWidth} className="border-[#2B2B2B]/24 cursor-default pointer-events-none">
              {connectedWalletDetails.icon && (
                <img src={connectedWalletDetails.icon} alt={`${connectedWalletDetails.name} icon`} className="inline-block w-6 h-6 mr-2" />
              )}
              <span className="truncate">{connectedWalletDetails.changeAddress || connectedWalletDetails.name}</span>
            </Button>
          )
          : (
            <Button type="button" variant={variant} fullWidth={fullWidth} loading={isConnecting}>
              {!isConnecting
                ? (
                  'Connect wallet'
                )
                : (
                  <>
                    Connecting wallet <SpinnerIcon className="animate-spin" />
                  </>
                )}
            </Button>
          )}
      </MenuButton>
      <MenuItems anchor="bottom" className="[--anchor-gap:8px]">
        {availableWallets.map(wallet => (
          <MenuItem key={wallet.key}>
            <button
              type="button"
              className="flex items-center cursor-pointer min-w-20 px-4 py-2"
              onClick={() => connectWallet(wallet.key)}
            >
              {wallet.icon && (
                <img src={wallet.icon} alt={`${wallet.name} icon`} className="inline-block w-6 h-6 mr-2" />
              )}
              <span>{wallet.name}</span>
            </button>
          </MenuItem>
        ))}
      </MenuItems>
    </Menu>
  );
}
