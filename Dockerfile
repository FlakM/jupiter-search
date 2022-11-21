FROM rust as builder
WORKDIR app


RUN apt-get update && \
    apt-get install -y libclang-dev openssl libssl-dev pkg-config

COPY . .

# to enable caching you might change commented out lines and run docker build using:
#   DOCKER_BUILDKIT=1 docker build -t podcast2text .
#
#RUN --mount=type=cache,target=/usr/local/cargo/registry \
#    --mount=type=cache,target=/home/root/app/target \
#    cargo build --release -p podcast2text
RUN cargo build --release -p podcast2text

FROM ubuntu:20.04 as runtime

WORKDIR app

RUN apt-get update && apt-get upgrade -y && \
    apt-get install -y ffmpeg openssl


#RUN --mount=type=cache,target=/home/root/app/target cp /home/root/app/target/release/podcast2text /usr/local/bin
COPY --from=builder /app/target/release/podcast2text /usr/local/bin
CMD ["/usr/local/bin/podcast2text"]
