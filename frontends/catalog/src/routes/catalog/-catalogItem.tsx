import { Link } from '@tanstack/react-router';

// Data
import { getCategoryLabel } from '~/data/category';

// Components
import { CalendarDueIcon, FlareIcon } from '~/components/icons';

interface Props {
  item: CatalogItem;
}

function ItemInfo({ item }: { item: Props['item']; }) {
  return (
    <>
      <div className="grow flex flex-row gap-3 p-4">
        <img src={item.icon} alt={`${item.name} Logo`} className="size-10.5" />
        <div>
          <h3 className="text-lg text-zinc-800 font-semibold capitalize">{item.name}</h3>
          <p className="text-sm text-zinc-600 mt-1.5">{item.description}</p>
        </div>
      </div>
      <div className="border-t border-zinc-200 px-4 py-3 flex flex-row items-center justify-between">
        <span className="text-xs text-zinc-500 font-medium rounded-full py-0.5 px-2 border-[0.5px] border-zinc-400">
          {getCategoryLabel(item.category)}
        </span>
        {item.comingSoon
          ? (
            <span className="flex items-center gap-1 text-sm text-zinc-400">
              <CalendarDueIcon className="size-4" />
              Planned
            </span>
          )
          : (
            <span className="flex items-center gap-1 text-sm text-[#0000FF]">
              Released
              <FlareIcon className="size-4" />
            </span>
          )}
      </div>
    </>
  );
}

export function CatalogItem({ item }: Props) {
  const classNames = 'w-full rounded-xl border border-zinc-200 bg-zinc-50 flex flex-col';
  if (item.comingSoon) {
    return (
      <div className={classNames}>
        <ItemInfo item={item} />
      </div>
    );
  }

  return (
    <Link
      to={!item.beta ? '/catalog/$category/$node' : '/'}
      params={{ category: item.category, node: item.slug }}
      hash={!item.beta ? undefined : 'beta'}
      className={`${classNames} hover:bg-[#0000FF]/2 hover:border-[#0000FF]/40 transition-colors duration-200 ease-in-out`}
    >
      <ItemInfo item={item} />
    </Link>
  );
}
