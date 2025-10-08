import { Buffer } from 'buffer';
import { useEffect, useState } from 'react';
import { createFileRoute, useNavigate } from '@tanstack/react-router';
import { decode } from 'cbor2';
import clsx from 'clsx';

// Components
import { ConnectWallet } from '~/components/wizard/ConnectWallet';
import { Button } from '~/components/ui/Button';
import { Callout } from '~/components/ui/Callout';

// Context
import { useWallet } from '~/contexts/wallet';
import { useWizard } from '~/contexts/wizard';

export const Route = createFileRoute('/_steps/')({
  component: RouteComponent,
});

function RouteComponent() {
  const { connectedWallet, connectedWalletDetails } = useWallet();
  const [utxos, setUtxos] = useState<UtxoRef[]>([]);
  const { setBadgeStatus } = useWizard();
  const [selectedUtxo, setSelectedUtxo] = useState<UtxoRef | null>(null);
  const navigate = useNavigate();

  const searchUtxos = async () => {
    if (!connectedWallet) return;

    const fetchedUtxos = await connectedWallet.getUtxos();
    const decodedUtxos = fetchedUtxos.map(utxoCbor => {
      const [ref] = decode(utxoCbor) as any;
      const [txHashBytes, index] = ref as [Uint8Array, number];
      const txHash = Buffer.from(txHashBytes).toString('hex');
      return { txHash, index };
    });

    setUtxos(decodedUtxos);
  };

  useEffect(() => {
    if (connectedWallet) {
      setBadgeStatus('In-progress');
      searchUtxos();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [connectedWallet]);

  if (!connectedWalletDetails) {
    return (
      <>
        <h2 className="text-[22px] text-[#42434D]">
          Connect with the wallet that will sign and submit the registration.
        </h2>

        <ConnectWallet variant="solid" className="mt-9.75" />
      </>
    );
  }

  return (
    <>
      <div>
        <label className="text-lg font-medium text-[#404D61]">Wallet address</label>
        <div className="flex items-center gap-8 mt-4">
          <input
            defaultValue={connectedWalletDetails.changeAddress ?? ''}
            readOnly
            className="w-full px-4 py-3 rounded-lg bg-[#F9F9F9] border border-[#E1E3E6] text-[#757D8A]"
          />
          <button className="underline underline-offset-2 text-[#2B2B2B] min-w-max">Change wallet</button>
        </div>
      </div>

      <div className="mt-14">
        <div className="text-[22px] text-[#2B2B2B]">Select UTxO for registration</div>
        <ul className="border-[0.5px] border-[#E1E3E6] rounded-lg p-6 mt-8">
          {utxos.length === 0 && <li className="mt-4 text-[#757D8A]">No UTxOs found in the connected wallet.</li>}
          {utxos.map(utxo => {
            const utxoRef = `${utxo.txHash}#${utxo.index}`;
            const isSelected = selectedUtxo?.index === utxo.index && selectedUtxo?.txHash === utxo.txHash;
            return (
              <li
                key={utxoRef}
                className={clsx(
                  'text-[#2B2B2B] border-b-[0.5px] last:border-b-0 border-[#CBD5E1] flex justify-between items-center py-3.5',
                  !isSelected && 'cursor-pointer hover:bg-[#F8F8FF]',
                )}
                onClick={() => setSelectedUtxo(utxo)}
              >
                <span className={selectedUtxo && !isSelected ? 'opacity-40' : ''}>{utxoRef}</span>
                <div
                  className={clsx(
                    'py-0.5 px-4.25 border-[0.5px] font-medium text-xs rounded-md',
                    isSelected && 'bg-[#F5FFF7] border-[#69C876]/50 text-[#69C876]',
                    !isSelected && 'bg-[#F8F8FF] border-[#0600FF]/50 text-[#0000FF]',
                  )}
                >
                  Select
                </div>
              </li>
            );
          })}
        </ul>
      </div>

      <Callout type="warning" className="mt-8">
        Please do not spend this UTxO, it needs to be consumed by the registration transaction.
      </Callout>

      <div className="flex justify-end mt-14">
        <Button
          type="button"
          disabled={!selectedUtxo}
          onClick={() => {
            navigate({
              to: '/2',
              state: { selectedUtxo },
            });
          }}
        >Continue
        </Button>
      </div>
    </>
  );
}
