import { Outlet, createRootRoute } from '@tanstack/react-router';
import { TanStackRouterDevtools } from '@tanstack/react-router-devtools';

// Components
import { Header } from '~/components/Header';

export const Route = createRootRoute({
  component: WizardRoot,
});

function WizardRoot() {
  return (
    <>
      <Header />
      <Outlet />
      <TanStackRouterDevtools />
    </>
  );
}
