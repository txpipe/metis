import { createServerFn } from '@tanstack/react-start';

// Utils
import { getBlobDetails, searchWorkloads } from '~/utils/registry';

export const getAvailableWorkloads = createServerFn({
  method: 'GET',
}).handler(async (): Promise<RegistryWorkload[]> => {
  const workloadsImages = await searchWorkloads();

  // Zot Search GraphQL
  const zotRepos = workloadsImages.data?.GlobalSearch.Repos ?? [];

  const output: RegistryWorkload[] = [];

  for (const repo of zotRepos) {
    const firstConfig = repo?.NewestImage?.Manifests?.[0]?.ConfigDigest;
    if (firstConfig && repo.Name) {
      const config = await getBlobDetails<RegistryWorkload['config']>(repo.Name, firstConfig).catch();

      if (!config) continue;

      output.push({
        repo: repo.Name,
        config,
      });
    }
  }

  return output;
});
