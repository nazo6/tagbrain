# build frontend
FROM node:20-slim AS frontend
RUN npm i -g pnpm
COPY ./frontend ./app/frontend
COPY ./package.json ./app/package.json
COPY ./pnpm-lock.yaml ./app/pnpm-lock.yaml
COPY ./pnpm-workspace.yaml ./app/pnpm-workspace.yaml
WORKDIR /app/frontend
RUN pnpm --version
RUN pnpm install --frozen-lockfile
RUN pnpm build

# build rust binary
FROM clux/muslrust:stable AS planner
RUN cargo install cargo-chef
COPY ./backend .
RUN cargo chef prepare --recipe-path recipe.json

FROM clux/muslrust:stable AS cacher
RUN cargo install cargo-chef
COPY --from=planner /volume/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

FROM clux/muslrust:stable AS builder
COPY ./backend /backend
WORKDIR /backend
COPY --from=cacher /volume/target target
COPY --from=cacher /root/.cargo /root/.cargo
COPY --from=frontend /app/frontend/dist /frontend/dist
RUN cargo build --release --target x86_64-unknown-linux-musl


# build final image
FROM alpine:latest

RUN apk add chromaprint
COPY --from=builder /backend/target/x86_64-unknown-linux-musl/release/tagbrain .
ENV CONFIG_PATH=/config/config.toml

ENTRYPOINT [ "/tagbrain" ]
