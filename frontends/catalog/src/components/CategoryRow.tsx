import { createElement } from 'react';
import { twMerge } from 'tailwind-merge';
import { Link, useNavigate } from '@tanstack/react-router';

// Data
import { categories } from '~/data/category';

// Icons
import * as icons from '~/components/icons';

interface Props {
  className?: string;
  activeCategory?: string | null;
}

export function CategoryRow({ className, activeCategory }: Props) {
  const navigate = useNavigate();

  return (
    <div className={twMerge('flex flex-row items-center mx-auto gap-6 w-fit max-w-full overflow-x-auto', className)}>
      {categories.map(category => {
        const isActive = activeCategory === category.value;
        return (
          <Link
            to="/catalog"
            search={{ category: category.value }}
            data-active={isActive}
            key={category.value}
            className="flex flex-col gap-1.5 text-sm font-medium text-zinc-500 items-center justify-center border border-zinc-200 rounded-md p-3 min-w-31 data-[active=true]:bg-[#0000FF]/2 data-[active=true]:border-[#0000FF]/40 data-[active=true]:text-[#0000FF] cursor-pointer"
            onClick={e => {
              if (isActive) {
                e.preventDefault();
                navigate({ to: '/catalog' });
              }
            }}
          >
            {createElement(icons[category.icon], { className: 'size-6' })}
            <span className="text-nowrap">{category.label}</span>
          </Link>
        );
      })}
    </div>
  );
}
