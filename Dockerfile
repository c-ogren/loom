# syntax=docker/dockerfile:1
ARG RUST_VERSION=1.91

FROM rust:${RUST_VERSION} AS build
WORKDIR /app

COPY .cargo /app/.cargo
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Online sqlx macros: point to a reachable DB from the build container.
# On macOS, host.docker.internal maps to your host; use the host port you exposed (e.g., 3307).
ENV DATABASE_URL="mysql://root:example@host.docker.internal:3307/c_oauth2"

RUN cargo build --locked --release
RUN install -Dm755 ./target/release/c-oauth2 /bin/server

FROM debian:bookworm-slim AS final
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
ARG UID=10001
RUN useradd -u "${UID}" -r -s /usr/sbin/nologin appuser
RUN mkdir -p ./logs && chown -R appuser:appuser ./logs
USER appuser
COPY --from=build /bin/server /bin/server
EXPOSE 3000
CMD ["/bin/server"]