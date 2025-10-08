import { createFileRoute, Link, useNavigate, useRouterState } from '@tanstack/react-router';
import { useMemo, useState } from 'react';

// Components
import { ArrowLeftIcon } from '~/components/icons/ArrowLeftIcon';
import { CopyIcon } from '~/components/icons/CopyIcon';
import { InfoCircleIcon } from '~/components/icons/InfoCircleIcon';
import { Button } from '~/components/ui/Button';

export const Route = createFileRoute('/_steps/2')({
  component: RouteComponent,
});

const MESSAGE_TO_SIGN = 'd8799fd8799fd8799f582046876a2250ec0e523eccc30b0fc6d6fa55c61dd200b83140acaab291edeb0b11ff00ff582103de3d4544b1789d22aae9b4ae24d9e85282a300f690f7c0a534434f7f62362153d8799fd8799f5820$REGISTRATION_HASH$ff$REGISTRATION_INDEX$ffff';
function RouteComponent() {
  const state = useRouterState({ select: s => s.location.state });
  const navigate = useNavigate();
  const [formValues, setFormValues] = useState({ signature: '', publicKey: '' });
  const selectedUtxo: UtxoRef | null = state?.selectedUtxo || null;

  const registrationMessage = useMemo(() => {
    return MESSAGE_TO_SIGN
      .replace('$REGISTRATION_HASH$', selectedUtxo ? selectedUtxo.txHash : '9ad6ff292db8408a53efc8bf2a9e6815cdc430185a7716cd3ecfc3f81a88f2d6')
      .replace('$REGISTRATION_INDEX$', `${selectedUtxo ? selectedUtxo.index : 0}`);
  }, [selectedUtxo]);

  return (
    <>
      <h2 className="text-[22px] text-[#42434D]">
        Sign the registration with your SPO key.
      </h2>

      <div className="mt-10">
        <div className="flex justify-between items-center text-[#404D61] font-medium">
          <div className="flex gap-1.5 items-center">
            Registration message
            <InfoCircleIcon />
          </div>
          <button
            type="button"
            className="flex gap-2 items-center text-sm cursor-pointer"
            onClick={() => {
              navigator.clipboard.writeText(registrationMessage);
            }}
          >
            Copy <CopyIcon className="w-5 h-5" strokeWidth={1.5} />
          </button>
        </div>
        <div className="mt-4 px-4 py-3 text-[#565656] bg-[#F9F9F9] border border-black/8 rounded-lg wrap-anywhere">
          {registrationMessage}
        </div>
      </div>

      <div className="mt-10">
        <div className="flex gap-1.5 items-center text-[#404D61] font-medium">
          Signature
          <InfoCircleIcon />
        </div>
        <textarea
          className="mt-4 bg-white border border-[#E1E3E6] py-3 px-4 rounded-lg w-full resize-none min-h-27 placeholder:text-[#757D8A]"
          placeholder="Please enter the signature generated with your SPO cold signing key."
          onChange={e => setFormValues({ ...formValues, signature: e.target.value })}
        />
      </div>

      <div className="mt-10">
        <div className="flex gap-1.5 items-center text-[#404D61] font-medium">
          Public key
          <InfoCircleIcon />
        </div>
        <textarea
          className="mt-4 bg-white border border-[#E1E3E6] py-3 px-4 rounded-lg w-full resize-none min-h-27 placeholder:text-[#757D8A]"
          placeholder="Please enter your SPO public key."
          onChange={e => setFormValues({ ...formValues, publicKey: e.target.value })}
        />
      </div>

      <div className="flex justify-between items-center mt-14">
        <Link to="/" className="flex items-center gap-1.25 text-[#2B2B2B]">
          <ArrowLeftIcon className="w-4 h-4" />
          Back
        </Link>

        <Button
          type="button"
          disabled={!selectedUtxo || !formValues.signature || !formValues.publicKey}
          onClick={() => {
            navigate({ to: '/3', state: { selectedUtxo, ...formValues } });
          }}
        >
          Continue
        </Button>
      </div>

    </>
  );
}
