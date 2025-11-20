// Components
import { CopyIcon, GitIcon } from '~/components/icons';
import { Card, CardBody, CardHeader } from '~/components/ui/Card';

// Utils
import dayjs from '~/utils/dayjs';

export function TabReadme({ item }: { item: CatalogItem; }) {
  return (
    <div className="grid grid-cols-1 md:grid-cols-[1fr_clamp(200px,40%,439px)] gap-12 mt-9">
      <Card>
        <CardHeader>Step 01</CardHeader>
        <CardBody className="flex flex-col gap-6 text-zinc-800">
          <p>
            To register as a Midnight block producer, you must first connect your Cardano wallet.
            This connection enables you to select a UTXO that will be consumed as part of the on-chain
            registration transaction. Please ensure your wallet contains sufficient ADA to cover the
            transaction fees before proceeding.
          </p>

          <img src="/images/wizard/midnight-01.png" alt="Wizard midnight step 01" className="w-full max-w-[610px]" />
        </CardBody>

        <CardHeader>Step 02</CardHeader>
        <CardBody className="flex flex-col gap-6 text-zinc-800">
          <p>
            After selecting a UTXO, you will be prompted to sign a registration message using your SPO
            (Stake Pool Operator) cold signing key. This cryptographic signature proves ownership of your
            stake pool. You must provide both the signature generated with your SPO cold key and your SPO
            public key. This step establishes the link between your Cardano stake pool and your Midnight
            block producer registration.
          </p>

          <img src="/images/wizard/midnight-02.png" alt="Wizard midnight step 02" className="w-full max-w-[610px]" />
        </CardBody>

        <CardHeader>Step 03</CardHeader>
        <CardBody className="flex flex-col gap-6 text-zinc-800">
          <p>
            Once your SPO signature is verified, the registration transaction will be generated and ready
            to submit on-chain. You will need to sign this transaction using the wallet you connected in
            Step 01. This transaction registers you as a candidate in the Midnight block producer committee.
            Please review the transaction details in your wallet before confirming and submitting it to the
            blockchain.
          </p>

          <img src="/images/wizard/midnight-03.png" alt="Wizard midnight step 03" className="w-full max-w-[610px]" />
        </CardBody>

        <CardHeader>Step 04</CardHeader>
        <CardBody className="flex flex-col gap-6 text-zinc-800">
          <p>
            Registration successful! Your transaction has been confirmed on-chain, and you are now registered
            as a candidate in the validator committee. The transaction hash (Tx Hash) is displayed for your
            records. You can access the Supernode Health Dashboard to monitor your Midnight block producer
            node's activity, performance metrics, and validator status. Your node will now be eligible to
            participate in block production on the Midnight network.
          </p>

          <img src="/images/wizard/midnight-04.png" alt="Wizard midnight step 04" className="w-full max-w-[610px]" />
        </CardBody>
      </Card>

      <Card>
        <CardHeader className="font-medium">{item.name}</CardHeader>
        <CardBody className="flex flex-col gap-8 text-zinc-900">
          <div className="flex flex-col gap-3">
            <p className="text-[#18181B]/50 text-lg/[1.2]">Author</p>
            <a
              href={item.author?.url ?? '#'}
              target="_blank"
              rel="noopener noreferrer"
              className="text-[#FF007F] text-lg/[1.2]"
            >@{item.author?.name}
            </a>
          </div>
          <div className="flex flex-col gap-3">
            <p className="text-[#18181B]/50 text-lg/[1.2]">Repository</p>
            <a
              href={item.repoUrl}
              className="w-fit flex items-center gap-2.5"
              target="_blank"
              rel="noopener noreferrer"
            >
              <GitIcon width="15" height="15" />
              <span className="underline">{item.repoUrl?.replace(/http(s)?:\/\//i, '')}</span>
            </a>
          </div>
          <div className="flex flex-col gap-3">
            <p className="text-[#18181B]/50 text-lg/[1.2]">Install</p>
            <div className="w-fit border border-[#18181B]/30 py-3 px-4.5 rounded-xl flex items-center gap-3 bg-white">
              <p className="font-mono wrap-anywhere break-all ">supernode install {item.ociName}</p>
              <button
                type="button"
                className="cursor-pointer"
                onClick={() => navigator.clipboard.writeText(`supernode install ${item.ociName}`)}
              >
                <CopyIcon className="size-5" />
              </button>
            </div>
          </div>
          <div className="flex flex-col gap-3">
            <p className="text-[#18181B]/50 text-lg/[1.2]">Version</p>
            <p className="">{item.version}</p>
          </div>
          {item.registryInfo?.Summary && (
            <div className="flex flex-col gap-3">
              <p className="text-[#18181B]/50 text-lg/[1.2]">Times deployed</p>
              <p className="">{item.registryInfo?.Summary?.DownloadCount}</p>
            </div>
          )}
          {item.registryInfo?.Images?.[0] && (
            <div className="flex flex-col gap-3">
              <p className="text-[#18181B]/50 text-lg/[1.2]">Publication date</p>
              <p className="">{dayjs(item.publishedDate ?? item.registryInfo?.Images?.[0].PushTimestamp).fromNow()}</p>
            </div>
          )}
        </CardBody>
      </Card>
    </div>
  );
}
