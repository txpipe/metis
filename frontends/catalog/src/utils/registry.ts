export const REGISTRY_URL = process.env.REGISTRY_ENDPOINT;

function isAvailable() {
  if (!REGISTRY_URL) {
    throw new Error('Registry not available');
  }
}

async function runGraphQLQuery<ResponseType>(path: string, query: string, variables: Record<string, any> = {}) {
  isAvailable();

  const response: { data?: ResponseType; } = await fetch(`${REGISTRY_URL!}/${path}`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ query, variables }),
  }).then(res => res.json());

  return response;
}

export async function searchWorkloads() {
  // This could be using a GraphQL Client, but in this case it's overkill
  const query = `
    query GlobalSearch($query: String!) {
      GlobalSearch(
          requestedPage: { limit: 100, offset: 0, sortBy: ALPHABETIC_DSC }
          query: $query
      ) {
          Repos {
              Name
              NewestImage {
                  Digest
                  Manifests {
                      ConfigDigest
                      Layers {
                          Digest
                      }
                  }
              }
          }
      }
    }
  `.trim();
  const variables = {
    query: 'extensions/',
  };

  const response = await runGraphQLQuery<{ GlobalSearch: GlobalSearchResult; }>('v2/_zot/ext/search', query, variables)
    .catch(() => ({ data: null }));

  return response;
}

export async function getManifestDetails(repoName: string, manifestDigest: string): Promise<Record<string, any>> {
  isAvailable();

  const path = `v2/${repoName}/manifests/${manifestDigest}`;
  const response = await fetch(`${REGISTRY_URL!}/${path}`).then(res => res.json());

  return response;
}

export async function getBlobDetails<T extends Record<string, any>>(repoName: string, blobDigest: string): Promise<T> {
  isAvailable();

  const path = `v2/${repoName}/blobs/${blobDigest}`;
  const response = await fetch(`${REGISTRY_URL!}/${path}`).then(res => res.json());

  return response;
}

export async function getManifestFromVersion(repoName: string, version: string): Promise<OciManifest | null> {
  isAvailable();

  const path = `v2/${repoName}/manifests/${version}`;
  const response = await fetch(`${REGISTRY_URL!}/${path}`).then(res => res.json());

  return response;
}

export async function getRepoInfo(repo: string) {
  const query = `
    query ExpandedRepoInfo($repo: String!) {
      ExpandedRepoInfo(repo: $repo) {
          Summary {
              DownloadCount
              StarCount
          }
          Images {
              DownloadCount
              PushTimestamp
          }
      }
    }
  `;
  const variables = {
    repo,
  };

  const response = await runGraphQLQuery<{ ExpandedRepoInfo: RepoInfo; }>('v2/_zot/ext/search', query, variables)
    .catch(() => ({ data: null }));

  return response;
}
