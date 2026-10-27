FROM node:26.12.1-bookworm-slim AS build
WORKDIR /app
COPY frontend/package.json frontend/package-lock.json frontend/.npmrc ./
RUN npm ci --ignore-scripts
COPY frontend/vite.config.ts frontend/tsconfig.json frontend/components.json ./
COPY frontend/src ./src
COPY frontend/static ./static
RUN npm run build

FROM caddy:2.11.4
COPY infra/production/Caddyfile /etc/caddy/Caddyfile
COPY infra/app-routes.caddy /etc/caddy/app-routes.caddy
COPY --from=build /app/build /srv
