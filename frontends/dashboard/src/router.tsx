import { createRouter } from '@tanstack/react-router';

// Import the generated route tree
import { routeTree } from './routeTree.gen';

// Create a new router instance
export const getRouter = () => {
  return createRouter({
    routeTree,
    scrollRestoration: true,
    defaultPreloadStaleTime: 0,
  });
};

// Register the router instance for type safety
declare module '@tanstack/react-router' {
  interface HistoryState {
    selectedUtxo?: UtxoRef | null;
    signature?: string | null;
    publicKey?: string | null;
  }
}
