declare interface CatalogItem {
  icon: string;
  name: string;
  slug: string;
  description: string;
  category: string;
  comingSoon: boolean;
  author?: {
    name: string;
    url?: string;
  };
  version?: string;
  repoUrl?: string;
  repoExtensionUrl?: string;
  helmResource?: string;
}
