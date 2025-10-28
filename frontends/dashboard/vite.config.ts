import { defineConfig } from 'vite';
import viteTsConfigPaths from 'vite-tsconfig-paths';
import { tanstackStart } from '@tanstack/react-start/plugin/vite';
import viteReact from '@vitejs/plugin-react';
import { nitroV2Plugin } from '@tanstack/nitro-v2-vite-plugin';
import devtoolsJson from 'vite-plugin-devtools-json';
import { devtools } from '@tanstack/devtools-vite';

import tailwindcss from '@tailwindcss/vite';

// https://vitejs.dev/config/
export default defineConfig({
  server: {
    port: 3000,
  },
  plugins: [
    devtoolsJson(),
    devtools({
      removeDevtoolsOnBuild: true,
    }),
    viteTsConfigPaths({
      projects: ['./tsconfig.json'],
    }),
    tailwindcss(),
    tanstackStart(),
    nitroV2Plugin(/*
      // nitro config goes here, e.g.
      { preset: 'node-server' }
    */),
    viteReact(),
  ],
});
