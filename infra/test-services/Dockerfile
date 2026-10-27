FROM rust:1.98.1-slim-bookworm AS build
WORKDIR /app
COPY . .
RUN cargo build --release --bin cloudflare_saas --bin echo_server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends curl && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/cloudflare_saas /usr/local/bin/cloudflare_saas
COPY --from=build /app/target/release/echo_server /usr/local/bin/echo_server
EXPOSE 8080
ENTRYPOINT ["cloudflare_saas"]
