// @ts-expect-error: FontSource doesn't have types but we don't need them
import '@fontsource-variable/inter';
import '@fontsource/poppins/400.css';
import '@fontsource/poppins/700.css';

import { useMemo } from 'react';
import { QueryClient } from '@tanstack/react-query';
import { HeadContent, Link, ScriptOnce, Scripts, createRootRouteWithContext } from '@tanstack/react-router';
import { TanStackRouterDevtoolsPanel } from '@tanstack/react-router-devtools';
import { ReactQueryDevtoolsPanel } from '@tanstack/react-query-devtools';
import { TanStackDevtools } from '@tanstack/react-devtools';

// Components
import { Header } from '~/components/Header';
import { Banner } from '~/components/ui/Banner';

// Utils
import { generateSocialMetadata } from '~/utils/social';

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
        title: 'SuperNode',
      },
      {
        name: 'description',
        content: 'Supercharge your blockchain infrastructure',
      },
      ...generateSocialMetadata(
        'SuperNode',
        'Supercharge your blockchain infrastructure',
        `${import.meta.env.VITE_PUBLIC_URL}/images/social/banner.jpg`,
        import.meta.env.VITE_PUBLIC_URL,
      ),
    ],
    links: [
      { rel: 'stylesheet', href: appCss },
      {
        rel: 'apple-touch-icon',
        sizes: '180x180',
        href: '/logo192.png',
      },
      {
        rel: 'icon',
        type: 'image/svg+xml',
        sizes: 'any',
        href: '/favicon.svg',
      },
      { rel: 'apple-touch-icon', href: '/apple-touch-icon.png' },
      { rel: 'manifest', href: '/manifest.json', color: '#fffff' },
      { rel: 'icon', href: '/favicon.ico' },
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
        <div>
          <Banner>
            We've released our first private beta version. <Link to="/" hash="beta" className="text-[#FF3F9F] underline underline-offset-2 text-nowrap">Sign-up</Link> to the waiting list.
          </Banner>
          <Header />
        </div>
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
