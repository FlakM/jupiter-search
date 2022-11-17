# syntax=docker/dockerfile:experimental
from rust
ENV HOME=/home/root
WORKDIR $HOME/app

COPY ./ $HOME/app

RUN apt-get update && \
    apt-get install -y libclang-dev libssl-dev pkg-config

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/home/root/app/target \
    cargo build --release -p downloader

FROM ubuntu:latest

RUN apt-get update && \
    apt-get install -y ffmpeg openssl

COPY --from=0 /home/root/app/target/release/downloader ./
CMD ["./downloader"]

