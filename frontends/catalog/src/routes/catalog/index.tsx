import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

// Data
import { useState } from 'react';
import { items } from '~/data/catalog';

// Components
import { Container } from '~/components/ui/Container';
import { CategoryRow } from '~/components/CategoryRow';

// Local
import { CatalogItem } from './-catalogItem';
import { SearchInput } from './-searchInput';

const catalogSearchSchema = z.object({
  category: z.string().optional(),
  query: z.string().optional(),
});

export const Route = createFileRoute('/catalog/')({
  validateSearch: catalogSearchSchema,
  loader: async () => {
    return {
      catalogItems: items,
    };
  },
  component: CatalogPage,
});

function CatalogPage() {
  const { category, query } = Route.useSearch();
  const { catalogItems } = Route.useLoaderData();
  const [searchText, setSearchText] = useState<string | null>(query || null);

  const filteredItems = (!category && !searchText)
    ? catalogItems
    : catalogItems.filter(item => {
      if (category && item.category !== category) {
        return false;
      }
      if (searchText) {
        const lowerSearchText = searchText.toLowerCase();
        return item.name.toLowerCase().includes(lowerSearchText);
      }
      return true;
    });

  return (
    <Container>
      <div className="flex flex-col items-center gap-6">
        <h1 className="text-4xl font-semibold text-zinc-800">Catalog</h1>
        <p className="text-zinc-600">Curated catalog of production-ready blockchain workloads.</p>
        <SearchInput onSearchText={setSearchText} />
      </div>

      <CategoryRow className="gap-6 mt-12" activeCategory={category} />

      <div className="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-[repeat(3,minmax(200px,421px))] w-fit gap-8 mx-auto mt-12">
        {filteredItems.map(item => (
          <CatalogItem key={item.slug} item={item} />
        ))}
      </div>
    </Container>
  );
}
