FROM alpine:latest

RUN apk add chromaprint
COPY ./backend/target/x86_64-unknown-linux-musl/release/tagbrain .
ENV CONFIG_PATH=/config/config.toml

ENTRYPOINT [ "/tagbrain" ]
