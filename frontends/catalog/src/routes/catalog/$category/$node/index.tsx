import { createFileRoute, Link, redirect } from '@tanstack/react-router';
import { z } from 'zod';

// Data
import { getItemBySlug } from '~/data/catalog';
import { getCategoryLabel } from '~/data/category';

// Components
import { Container } from '~/components/ui/Container';
import { Breadcrumb } from '~/components/ui/Breadcrumb';
import { button } from '~/components/ui/Button';
// import { Stack2Icon } from '~/components/icons';
// import { TabItem, Tabs } from '~/components/ui/Tabs';
// import { InfoCircleIcon } from '~/components/icons/InfoCircleIcon';

// Local
import { TabReadme } from './-tabs/readme';
// import { TabResources } from './-tabs/resources';

const validateSearch = z.object({
  tab: z.enum(['readme', 'resources']).optional(),
});

// type SearchStruct = z.infer<typeof validateSearch>;

export const Route = createFileRoute('/catalog/$category/$node/')({
  validateSearch,
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

// const tabs = [
//   { icon: <InfoCircleIcon className="size-5" />, value: 'readme', label: 'Readme' },
//   { icon: <Stack2Icon className="size-5" />, value: 'resources', label: 'Resources' },
// ];

function RouteComponent() {
  // const { tab = 'readme' } = Route.useSearch();
  const { item } = Route.useLoaderData();
  // const navigate = Route.useNavigate();

  // const onTabChanged = (index: number, value?: string) => {
  //   const realValue = value ?? tabs[index]?.value;

  //   if (realValue === tab) return;

  //   navigate({ search: old => ({ ...old, tab: realValue as SearchStruct['tab'] }) });
  // };

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
        <Link to="/" hash="beta" className={button({ className: 'min-w-47.5' })}>Install </Link>
      </div>

      {/* <Tabs
        className="-mx-4 px-4 sm:-mx-14 sm:px-14 mt-8"
        onTabChanged={onTabChanged}
        initialTab={tabs.findIndex(t => t.value === tab)}
      >
        {tabs.map(tabItem => (
          <TabItem
            key={tabItem.value}
            icon={tabItem.icon}
            value={tabItem.value}
            label={tabItem.label}
          />
        ))}
      </Tabs>

      {tab === 'readme' && <TabReadme item={item} />}
      {tab === 'resources' && <TabResources item={item} />} */}
      <TabReadme item={item} />
    </Container>
  );
}
