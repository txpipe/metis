// @ts-expect-error: FontSource doesn't have types but we don't need them
import '@fontsource-variable/inter';
import '@fontsource/poppins/400.css';
import '@fontsource/poppins/700.css';

import { QueryClient } from '@tanstack/react-query';
import { HeadContent, ScriptOnce, Scripts, createRootRouteWithContext } from '@tanstack/react-router';
import { TanStackRouterDevtoolsPanel } from '@tanstack/react-router-devtools';
import { ReactQueryDevtoolsPanel } from '@tanstack/react-query-devtools';
import { TanStackDevtools } from '@tanstack/react-devtools';

// Components
import { useMemo } from 'react';
import { Header } from '~/components/Header';

import appCss from '~/styles.css?url';

export const Route = createRootRouteWithContext<{
  queryClient: QueryClient;
}>()({
  head: () => ({
    meta: [
      {
        charSet: 'utf-8',
      },
      {
        name: 'viewport',
        content: 'width=device-width, initial-scale=1',
      },
      {
        title: 'SuperNode Catalog',
      },
    ],
    links: [
      { rel: 'stylesheet', href: appCss },
    ],
  }),

  shellComponent: RootDocument,
});

function RootDocument({ children }: { children: React.ReactNode; }) {
  const isMac = useMemo(() => {
    return /(Mac|iPhone|iPod|iPad)/i.test(navigator.userAgent);
  }, []);

  return (
    <html lang="en" className={isMac ? 'os-macos' : ''}>
      <head>
        <HeadContent />
      </head>
      <body>
        <Header />
        {children}
        <TanStackDevtools
          config={{
            position: 'bottom-right',
          }}
          plugins={[
            {
              name: 'Tanstack Router',
              render: <TanStackRouterDevtoolsPanel />,
            },
            {
              name: 'React Query',
              render: <ReactQueryDevtoolsPanel />,
            },
          ]}
        />
        <Scripts />
      </body>
      <ScriptOnce>
        {`
          try {
            if (/(Mac|iPhone|iPod|iPad)/i.test(navigator.userAgent)) {
              document.documentElement.classList.add('os-macos');
            }
          } catch (_) {}
          `}
      </ScriptOnce>
    </html>
  );
}
