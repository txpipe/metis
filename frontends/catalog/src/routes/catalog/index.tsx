import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

// Data
import { getCatalog } from '~/data/catalog';

// Components
import { Container } from '~/components/ui/Container';
import { CategoryRow } from '~/components/CategoryRow';

// Local
import { CatalogItem } from './-catalogItem';

const catalogSearchSchema = z.object({
  category: z.string().optional(),
  query: z.string().optional(),
});

export const Route = createFileRoute('/catalog/')({
  validateSearch: catalogSearchSchema,
  loader: async () => {
    const catalogItems = await getCatalog();
    return {
      catalogItems,
    };
  },
  component: CatalogPage,
});

function CatalogPage() {
  const { category, query } = Route.useSearch();
  const { catalogItems } = Route.useLoaderData();

  const filteredItems = (!category && !query)
    ? catalogItems
    : catalogItems.filter(item => {
      if (category && item.category !== category) {
        return false;
      }
      if (query) {
        const lowerSearchText = query.toLowerCase();
        return item.name.toLowerCase().includes(lowerSearchText);
      }
      return true;
    });

  return (
    <Container>
      <CategoryRow className="gap-6 mt-12" activeCategory={category} />

      <div className="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-[repeat(3,minmax(200px,421px))] w-fit gap-8 mx-auto mt-12">
        {filteredItems.map(item => (
          <CatalogItem key={item.slug} item={item} />
        ))}
      </div>
    </Container>
  );
}
