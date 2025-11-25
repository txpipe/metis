FROM node:22-alpine AS base
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable
WORKDIR /app

FROM base AS dev-deps
COPY package.json pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile

FROM base AS build
COPY --from=dev-deps /app/node_modules ./node_modules
COPY . .
ENV VITE_PUBLIC_URL="https://supernode.store"
RUN pnpm build

FROM base AS prod-deps
COPY package.json pnpm-lock.yaml ./
RUN pnpm install --prod --frozen-lockfile --ignore-scripts

FROM node:22-alpine AS runner
ENV NODE_ENV=production
WORKDIR /app
COPY --from=prod-deps /app/node_modules ./node_modules
COPY package.json pnpm-lock.yaml ./
COPY --from=build /app/.output ./
ENV VITE_PUBLIC_URL="https://supernode.store"
EXPOSE 3000
CMD ["node", "server/index.mjs"]
