import { Outlet, createRootRoute } from '@tanstack/react-router';
import { TanStackRouterDevtools } from '@tanstack/react-router-devtools';

// Components
import { Header } from '~/components/Header';

export const Route = createRootRoute({
  component: DashboardRoot,
});

function DashboardRoot() {
  return (
    <>
      <Header />
      <Outlet />
      <TanStackRouterDevtools />
    </>
  );
}
