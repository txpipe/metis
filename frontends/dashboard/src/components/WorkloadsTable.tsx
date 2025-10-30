import { useEffect, useRef, useState } from 'react';
import { type ColumnDef, flexRender, getCoreRowModel, getFilteredRowModel, useReactTable } from '@tanstack/react-table';
import clsx from 'clsx';

// Components
import { SearchIcon } from '~/components/icons/SearchIcon';
import { Button } from '~/components/ui/Button';
import { CircleCheckIcon } from '~/components/icons/CircleCheckIcon';

// Mock Data
import { workloads } from '~/data/workloads';

interface Props {
  onWorkloadSelected: (workload: Workload) => void;
  className?: string;
}

const memAddedState: Record<string, number> = {};

const switchAddedAfter = 2000; // ms

function RowActionsCell({
  workload,
  onWorkloadSelected,
}: {
  workload: Workload;
  onWorkloadSelected: Props['onWorkloadSelected'];
}) {
  let existingTimeout = useRef<NodeJS.Timeout | null>(null);
  const [added, setAdded] = useState(!!memAddedState[workload.id]);

  useEffect(() => {
    if (memAddedState[workload.id]) {
      let diff = switchAddedAfter - Math.max(Date.now() - memAddedState[workload.id], 0);
      existingTimeout.current = setTimeout(() => {
        setAdded(false);
        delete memAddedState[workload.id];
      }, diff);
    }

    return () => {
      if (existingTimeout.current) {
        clearTimeout(existingTimeout.current);
      }
    };
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <div className="flex justify-end">
      <Button
        type="button"
        color={added ? 'green' : 'blue'}
        size="small-auto-x"
        className={clsx('gap-0.5 w-18', { 'pointer-events-none': added })}
        disabled={added}
        onClick={() => {
          setAdded(true);
          memAddedState[workload.id] = Date.now();
          existingTimeout.current = setTimeout(() => {
            delete memAddedState[workload.id];
            setAdded(false);
          }, switchAddedAfter);
          onWorkloadSelected(workload);
        }}
      >
        {added
          ? (
            <>
              Added <CircleCheckIcon className="w-3 h-3" />
            </>
          )
          : 'Add'}

      </Button>
    </div>
  );
}

const createColumns = (onWorkloadSelected: Props['onWorkloadSelected']): ColumnDef<Workload>[] => [
  {
    header: 'Workload',
    accessorKey: 'name',
  },
  {
    header: 'Network',
    accessorKey: 'network',
  },
  {
    header: 'Description',
    accessorKey: 'description',
  },
  {
    header: 'ID',
    accessorKey: 'id',
    filterFn: 'includesString',
  },
  {
    id: 'actions',
    size: 90,
    cell: ({ row }) => {
      return <RowActionsCell workload={row.original} onWorkloadSelected={onWorkloadSelected} />;
    },
  },
];

export function WorkloadsTable({ onWorkloadSelected, className }: Props) {
  const columns = createColumns(onWorkloadSelected);

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
          type="number"
          name="search-id"
          placeholder="Search by ID"
          className="border-0 rounded-md shadow-[0px_2px_5px_0px_rgba(32,42,54,0.06),0px_1px_5px_-4px_rgba(19,19,22,0.4),0px_0px_0px_1px_rgba(33,33,38,0.1)] py-1.75 pl-10 pr-3 w-62.5"
          min={0}
          onChange={e => {
            const value = e.target.valueAsNumber;
            table.setColumnFilters(current => {
              const newFilters = current.filter(filter => filter.id !== 'id');
              if (value && !isNaN(value)) {
                newFilters.push({ id: 'id', value });
              }
              return newFilters;
            });
          }}
        />
      </div>
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
    </div>
  );
}
