FROM node:22-alpine AS dependencies-env
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable
COPY . /app

FROM dependencies-env AS development-dependencies-env
WORKDIR /app
RUN pnpm i --frozen-lockfile

FROM dependencies-env AS production-dependencies-env
COPY ./package.json ./pnpm-lock.yaml /app/
WORKDIR /app
RUN pnpm i --prod --frozen-lockfile --ignore-scripts

FROM dependencies-env AS build-env
COPY --from=development-dependencies-env /app/node_modules /app/node_modules
WORKDIR /app
RUN pnpm build

FROM nginx:alpine
COPY --from=build-env /app/dist /usr/share/nginx/html
COPY ./nginx/nginx.conf /etc/nginx/nginx.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
