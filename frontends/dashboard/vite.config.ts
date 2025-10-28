import { defineConfig } from 'vite';
import { tanstackStart } from '@tanstack/react-start/plugin/vite';
import viteReact from '@vitejs/plugin-react';
import viteTsConfigPaths from 'vite-tsconfig-paths';
import tailwindcss from '@tailwindcss/vite';
import { nitroV2Plugin } from '@tanstack/nitro-v2-vite-plugin';
import { devtools } from '@tanstack/devtools-vite';
import devtoolsJson from 'vite-plugin-devtools-json';

const config = defineConfig({
  server: {
    port: 3000,
  },
  plugins: [
    devtoolsJson(),
    devtools({
      removeDevtoolsOnBuild: true,
    }),
    // this is the plugin that enables path aliases
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

export default config;
