FROM node:26.12.1-bookworm-slim AS build
WORKDIR /app
COPY frontend/package.json frontend/package-lock.json frontend/.npmrc ./
RUN npm ci --ignore-scripts
COPY metadata.toml /metadata.toml
COPY frontend/vite.config.ts frontend/vite-plugin-toml.js frontend/tsconfig.json frontend/components.json ./
COPY frontend/src ./src
COPY frontend/static ./static
ARG FORTROPP_VERSION=dev
ENV FORTROPP_VERSION=$FORTROPP_VERSION
RUN npm run build

FROM caddy:2.11.4
COPY infra/production/Caddyfile /etc/caddy/Caddyfile
COPY infra/app-routes.caddy /etc/caddy/app-routes.caddy
COPY --from=build /app/build /srv
