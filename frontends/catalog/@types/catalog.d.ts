declare interface CatalogItem {
  icon: string;
  name: string;
  slug: string;
  description: string;
  category: string;
  comingSoon: boolean;
  beta?: boolean;
  author?: {
    name: string;
    url?: string;
  };
  version?: string;
  repoUrl?: string;
  ociName?: string;
  registryInfo?: RepoInfo;
  publishedDate?: string;
}
