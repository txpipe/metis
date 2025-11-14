import { createFileRoute, Link, redirect } from '@tanstack/react-router';

// Data
import { getItemBySlug } from '~/data/catalog';
import { getCategoryLabel } from '~/data/category';

// Components
import { Container } from '~/components/ui/Container';
import { Breadcrumb } from '~/components/ui/Breadcrumb';
import { Button } from '~/components/ui/Button';
import { Card, CardBody, CardHeader } from '~/components/ui/Card';
import { CopyIcon, GitIcon, StarIcon } from '~/components/icons';

export const Route = createFileRoute('/catalog/$category/$node/')({
  loader: async ({ params: { node } }) => {
    const item = await getItemBySlug({ data: { slug: node } });
    if (!item || item.comingSoon) {
      throw redirect({ to: '/catalog' });
    }
    return {
      item,
    };
  },
  component: RouteComponent,
});

function RouteComponent() {
  const { item } = Route.useLoaderData();
  return (
    <Container>
      <Breadcrumb>
        <Link className="text-sm md:text-base" to="/catalog">Catalog</Link>
        <Link className="text-sm md:text-base" to="/catalog" search={{ category: item.category }}>{getCategoryLabel(item.category)}</Link>
        <span className="text-zinc-800 font-semibold text-sm md:text-base">{item.name}</span>
      </Breadcrumb>

      <div className="grid items-start grid-cols-1 md:grid-cols-[1fr_auto] gap-8 mt-6">
        <div className="flex flex-row gap-4 items-start">
          <img src={item.icon} alt={`${item.name} logo`} className="size-15.5" />
          <div className="grow">
            <h2 className="text-3xl/[40px] font-semibold text-zinc-800">{item.name}</h2>
            <p className="text-sm/[20px] text-zinc-500">{item.description}</p>
            <div className="mt-2 text-xs w-fit text-zinc-400 font-medium rounded-full py-0.5 px-2 border-[0.5px] border-zinc-400">
              {getCategoryLabel(item.category)}
            </div>
          </div>
        </div>
        <Button type="button" className="gap-1.5 min-w-47.5">Install <StarIcon className="size-4" /></Button>
      </div>

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

            <img src="/images/wizard/midnight-01.png" alt="Wizard midnight step 01" className="w-full max-w-[610px] shadow-[0px_0px_16px_0px_rgba(0,0,0,0.08)]" />
          </CardBody>
        </Card>

        <Card>
          <CardHeader>{item.name}</CardHeader>
          <CardBody className="flex flex-col gap-8 text-zinc-900">
            <div className="flex flex-col gap-3">
              <p className="text-[#18181B]/50 text-lg/[1.2]">Author</p>
              <a
                href={item.author?.url ?? '#'}
                target="_blank"
                rel="noopener noreferrer"
                className="text-[#0000FF] text-lg/[1.2]"
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
              <p className="text-[#18181B]/50 text-lg/[1.2]">Version</p>
              <p className="">{item.version}</p>
            </div>
            <div className="flex flex-col gap-3">
              <p className="text-[#18181B]/50 text-lg/[1.2]">Install</p>
              <p className="w-fit border border-[#18181B]/30 py-3 px-4.5 rounded-xl flex items-start gap-2">
                <p className="font-mono wrap-anywhere">helm install [namespace] {item.helmResource}</p>
                <button
                  type="button"
                  className="cursor-pointer"
                  onClick={() => navigator.clipboard.writeText(`helm install [namespace] ${item.helmResource}`)}
                >
                  <CopyIcon className="size-5" />
                </button>
              </p>
            </div>
          </CardBody>
        </Card>
      </div>
    </Container>
  );
}
