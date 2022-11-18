# syntax=docker/dockerfile:experimental
from rust as builder
ENV HOME=/home/root
WORKDIR $HOME/app

COPY ./ $HOME/app

RUN apt-get update && \
    apt-get install -y libclang-dev openssl libssl-dev pkg-config

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/home/root/app/target \
    cargo build --release -p downloader

RUN --mount=type=cache,target=/home/root/app/target ls -al /home/root/app/ta && exit 1 

FROM ubuntu:20.04


RUN apt-get update && apt-get upgrade && \
    apt-get install -y ffmpeg openssl

RUN --mount=type=cache,target=/home/root/app/target cp /home/root/app/target/release/downloader .

USER 1000

CMD ["./downloader"]

