# syntax=docker/dockerfile:1

# Pinned by digest, with the tag beside it for Dependabot (.github/dependabot.yml), which moves the pin.
FROM rust:1-alpine@sha256:8d0e82abfdf9a38071e07cabc5afba4ddbe077631a80e4e742a760c38beadc33 AS build
RUN apk add --no-cache musl-dev
WORKDIR /src

# Dependencies first, so editing finstats itself doesn't rebuild the world. The two cache mounts make
# even that incremental between builds on one machine: the crate registry and the target directory
# survive, so a changed source file recompiles finstats and relinks instead of building everything
# again, which takes minutes off every local rebuild and every run of the QA image check. They are BuildKit
# caches and change nothing about the image: the binary is copied out of the cache onto a real layer,
# and a builder without them (a cold CI runner) simply builds as before.
COPY Cargo.toml Cargo.lock ./
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=finstats-registry \
    --mount=type=cache,target=/src/target,id=finstats-target \
    mkdir -p src web && echo 'fn main() {}' > src/main.rs \
    && cargo build --release --locked \
    && rm -rf src target/release/deps/finstats-* target/release/finstats

COPY src ./src
COPY web ./web
# The typeface the shareable cards are drawn in (the web UI's own is woff2, which the renderer cannot read).
COPY fonts ./fonts
# Compiled into the binary: the in-app patch notes, and the licences: finstats' own and
# the notice generated for every crate it is built on.
COPY CHANGELOG.md LICENSE THIRD-PARTY.json ./
# The Patch notes in every other language, compiled in beside CHANGELOG.md.
COPY changelog ./changelog
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=finstats-registry \
    --mount=type=cache,target=/src/target,id=finstats-target \
    cargo build --release --locked && cp target/release/finstats /finstats

FROM alpine:3.22@sha256:49340b346798875b5592e5d420241f2f9cfba5f47e62ebb1446a1b34734e898f
# Ties the published image to its source; the release workflow adds version, revision and date.
LABEL org.opencontainers.image.title="finstats" \
      org.opencontainers.image.description="Lightweight playback statistics for Jellyfin" \
      org.opencontainers.image.source="https://github.com/finstats/finstats" \
      org.opencontainers.image.licenses="GPL-3.0-only"
# tzdata: "plays per day" and the hour-of-day heatmap follow the TZ variable.
# su-exec: the entrypoint drops from root to the finstats user with it.
# apk upgrade: Alpine fixes a CVE in its repositories before a new base image carries it (zlib 1.4-r0 against
# CVE-2026-4738, while alpine:3.22.8 still shipped 1.3.2-r1), so the image takes every fix there is when it is built.
RUN apk upgrade --no-cache \
    && apk add --no-cache tzdata su-exec \
    && addgroup -g 1000 finstats && adduser -D -u 1000 -G finstats finstats \
    && mkdir /data && chown finstats:finstats /data
COPY --from=build /finstats /usr/local/bin/finstats
COPY docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh

# No USER line on purpose: the entrypoint starts as root only to make /data writable (Docker creates a
# missing bind-mount folder as root), then runs finstats as PUID:PGID, 1000:1000 unless told otherwise.
ENV FINSTATS_DATA_DIR=/data \
    FINSTATS_BIND=0.0.0.0:8080
VOLUME /data
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s \
    CMD wget -qO /dev/null http://127.0.0.1:8080/api/status || exit 1
ENTRYPOINT ["docker-entrypoint.sh"]
