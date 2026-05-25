# syntax=docker/dockerfile:1

FROM node:22-bookworm-slim AS web-build
WORKDIR /src/clients/web
COPY clients/web/package.json clients/web/package-lock.json ./
RUN npm ci
COPY clients/web/ ./
RUN npm run build

FROM rust:1.88-bookworm AS rust-build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY sdk ./sdk
COPY clients/desktop/src-tauri ./clients/desktop/src-tauri
RUN cargo build --release -p ygg-cli --bin ygg

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates git \
  && rm -rf /var/lib/apt/lists/* \
  && useradd --system --create-home --home-dir /home/ygg --shell /usr/sbin/nologin ygg \
  && mkdir -p /app/public /data \
  && chown -R ygg:ygg /app /data

COPY --from=rust-build /src/target/release/ygg /usr/local/bin/ygg
COPY --from=web-build /src/clients/web/dist /app/public
COPY packages/official/git-tools-lab /app/packages/official/git-tools-lab
COPY packages/official/integrity-lab /app/packages/official/integrity-lab
COPY packages/official/install-lab /app/packages/official/install-lab
COPY packages/official/secret-store-lab /app/packages/official/secret-store-lab
COPY docker/entrypoint.sh /usr/local/bin/ygg-zeabur-entrypoint
RUN chmod +x /usr/local/bin/ygg-zeabur-entrypoint \
  && chown -R ygg:ygg /app

USER ygg
ENV PORT=8080 \
  YGG_DATA_DIR=/data \
  YGG_STATIC_DIR=/app/public \
  YGG_PROFILE=default
EXPOSE 8080
CMD ["/usr/local/bin/ygg-zeabur-entrypoint"]
