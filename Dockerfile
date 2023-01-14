FROM rust as builder
WORKDIR app


RUN apt-get update && \
    apt-get install -y libclang-dev openssl libssl-dev pkg-config

COPY . .

# It would have been quite fantastic to use buildkit as suggested here
# https://stackoverflow.com/a/60590697/5665181
# But this is undoable in github ci actions because of 
# cache mounts are not exported right now https://github.com/docker/build-push-action/issues/716
RUN cargo build --release -p podcast2text

FROM ubuntu:20.04 as runtime

WORKDIR app

RUN apt-get update && apt-get upgrade -y && \
    apt-get install -y ffmpeg openssl ca-certificates


COPY --from=builder /app/target/release/podcast2text /usr/local/bin

VOLUME /data/models /data/output

ENV MODEL_PATH=/data/models/model.bin
ENV OUTPUT_DIR=/data/output/

ENTRYPOINT [ "/usr/local/bin/podcast2text"]
