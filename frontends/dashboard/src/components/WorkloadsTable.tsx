import { ReactNode, useEffect, useRef, useState } from 'react';
import { type ColumnDef, flexRender, getCoreRowModel, getFilteredRowModel, useReactTable } from '@tanstack/react-table';
import clsx from 'clsx';
import { useServerFn } from '@tanstack/react-start';

// Components
import { SearchIcon } from '~/components/icons/SearchIcon';
import { Button } from '~/components/ui/Button';
import { CircleCheckIcon } from '~/components/icons/CircleCheckIcon';

// Utils
import { getAvailableWorkloads, addWorkload } from '~/utils/home/calls';

interface Props {
  onWorkloadSelected: (workload: RegistryWorkload) => void;
  className?: string;
}

const memAddedState: Record<string, number> = {};

const switchAddedAfter = 2000; // ms

function RowActionsCell({
  workload,
  onWorkloadSelected,
}: {
  workload: RegistryWorkload;
  onWorkloadSelected: Props['onWorkloadSelected'];
}) {
  let existingTimeout = useRef<NodeJS.Timeout | null>(null);
  const [added, setAdded] = useState(!!memAddedState[workload.repo]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (memAddedState[workload.repo]) {
      let diff = switchAddedAfter - Math.max(Date.now() - memAddedState[workload.repo], 0);
      if (diff < 0) {
        setAdded(false);
        delete memAddedState[workload.repo];
      } else {
        existingTimeout.current = setTimeout(() => {
          setAdded(false);
          delete memAddedState[workload.repo];
        }, diff);
      }
    }

    return () => {
      if (existingTimeout.current) {
        clearTimeout(existingTimeout.current);
      }
    };
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const handleAddWorkload = async () => {
    setLoading(true);
    try {
      const result = await addWorkload({
        data: {
          repo: workload.repo,
          name: workload.config.name,
          version: workload.config.version,
        },
      });
      if (result.success) {
        setAdded(true);
        memAddedState[workload.repo] = Date.now();
        existingTimeout.current = setTimeout(() => {
          delete memAddedState[workload.repo];
          setAdded(false);
        }, switchAddedAfter);
        onWorkloadSelected(workload);
      }
    } catch (err) {
      // eslint-disable-next-line no-console
      console.error('Error adding workload:', err);
    } finally {
      setLoading(false);
    }
  };

  let childrenComponent: ReactNode = 'Add';
  if (added) {
    childrenComponent = (
      <>
        Added <CircleCheckIcon className="w-3 h-3" />
      </>
    );
  } else if (loading) {
    childrenComponent = (
      <>
        Adding...
      </>
    );
  }

  return (
    <div className="flex justify-end">
      <Button
        type="button"
        color={added ? 'green' : 'blue'}
        size="small-auto-x"
        className={clsx('gap-0.5 w-18', { 'pointer-events-none': added })}
        disabled={added}
        onClick={handleAddWorkload}
        loading={loading}
      >
        {childrenComponent}
      </Button>
    </div>
  );
}

const createColumns = (onWorkloadSelected: Props['onWorkloadSelected']): ColumnDef<RegistryWorkload>[] => [
  {
    header: 'Workload',
    id: 'name',
    accessorKey: 'config.name',
    filterFn: 'includesString',
  },
  {
    header: 'Description',
    accessorKey: 'config.description',
  },
  {
    id: 'actions',
    size: 90,
    cell: ({ row }) => {
      return <RowActionsCell workload={row.original} onWorkloadSelected={onWorkloadSelected} />;
    },
  },
];

let workloadsDataCache: RegistryWorkload[] = [];

export function WorkloadsTable({ onWorkloadSelected, className }: Props) {
  const columns = createColumns(onWorkloadSelected);
  const workloadsData = useServerFn(getAvailableWorkloads);
  const [loading, setLoading] = useState(workloadsDataCache.length === 0);
  const [workloads, setWorkloads] = useState<RegistryWorkload[]>(workloadsDataCache);

  useEffect(() => {
    if (workloadsDataCache.length === 0) {
      setLoading(true);
      workloadsData().then(data => {
        setWorkloads(data);
        workloadsDataCache = data;
      }).catch().finally(() => {
        setLoading(false);
      });
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const table = useReactTable({
    data: workloads,
    columns,
    getCoreRowModel: getCoreRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    enableColumnFilters: true,
  });

  return (
    <div className={clsx('bg-white rounded-3xl p-6', className)}>
      <div className="relative">
        <SearchIcon className="absolute w-5 h-5 text-[#9394A1] pointer-events-none top-1/2 -translate-y-1/2 left-3" />
        <input
          type="text"
          name="search-id"
          placeholder="Search by Name"
          className="border-0 rounded-md shadow-[0px_2px_5px_0px_rgba(32,42,54,0.06),0px_1px_5px_-4px_rgba(19,19,22,0.4),0px_0px_0px_1px_rgba(33,33,38,0.1)] py-1.75 pl-10 pr-3 w-62.5"
          onChange={e => {
            const value = e.target.value;
            table.setColumnFilters(current => {
              const newFilters = current.filter(filter => filter.id !== 'name');
              if (value.trim()) {
                newFilters.push({ id: 'name', value });
              }
              return newFilters;
            });
          }}
        />
      </div>
      {loading
        ? (
          <div className="w-full flex justify-center items-center mt-8">
            <div className="w-6 h-6 border-4 border-zinc-400 border-t-transparent border-solid rounded-full animate-spin" />
          </div>
        )
        : (
          <table className="w-full table-auto mt-8">
            <thead>
              {table.getHeaderGroups().map(headerGroup => (
                <tr key={headerGroup.id} className="text-left border-b-[0.5px] border-[#CBD5E1] [&_th]:pt-2 [&_th]:pb-3">
                  {headerGroup.headers.map(header => (
                    <th key={header.id} colSpan={header.colSpan} style={header.id === 'actions' ? { width: header.getSize() } : {}}>
                      {header.isPlaceholder ? null : flexRender(header.column.columnDef.header, header.getContext())}
                    </th>
                  ))}
                </tr>
              ))}
            </thead>
            <tbody>
              {table.getRowModel().rows.map(row => (
                <tr key={row.id} className="[&_td]:py-2 border-b-[0.5px] border-[#CBD5E1]">
                  {row.getVisibleCells().map(cell => (
                    <td key={cell.id}>
                      {flexRender(cell.column.columnDef.cell, cell.getContext())}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        )}
    </div>
  );
}
