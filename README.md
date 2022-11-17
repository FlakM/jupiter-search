# jupiter-search

Showcase for indexing jupiter network podcasts using meilisearch.
This repository is build in order to provide possible solution to following problems:

- [search](https://github.com/JupiterBroadcasting/jupiterbroadcasting.com/issues/26)
- [transcription](https://github.com/JupiterBroadcasting/jupiterbroadcasting.com/issues/301)

**DISCLAIMER!**

Warning! This is a very dirty version to showcase how indexing/transcription might work.


## Building

To build you would need following packages on your system:

- cargo
- pkg-config
- openssl
- ffmpeg

There is a nix flake configured to ship build dependencies
just run `direnv allow` and run:

```shell
git submodule update --init --recursive
cargo build --release --examples
# if you are using nix os and want to use the binary you should fix the interpreter
patchelf --set-interpreter /usr/lib64/ld-linux-x86-64.so.2 target/release/examples/get_transcript
```

To appease the gods of good taste please add following pre commit hook:

```
git config --local core.hooksPath .githooks
```

## Installation

### Install meilisearch

```shell
docker pull getmeili/meilisearch:v0.29
docker run -it --rm \
    -p 7700:7700 \
    -e MEILI_MASTER_KEY='MASTER_KEY'\
    -v $(pwd)/meili_data:/meili_data \
    getmeili/meilisearch:v0.29 \
    meilisearch --env="development"
```

### Run index creation and data loading

`cargo run`


### Running inference of some audio

1. Download whisper model

```
mkdir models
# this might be one of:
# "tiny.en" "tiny" "base.en" "base" "small.en" "small" "medium.en" "medium" "large"
model=medium.en
curl --output models/ggml-$model.bin https://ggml.ggerganov.com/ggml-model-whisper-$model.bin
```
2. Download the example audio from rss feed

```
curl https://feed.jupiter.zone/link/19057/15745245/55bb5263-04be-43a3-8b92-678072a9cfc8.mp3 -L -o action.mp3
```

3. Install `ffmpeg` and put it on `PATH` variable.

4. Run the inference example

```
cargo run --release --example=get_transcript -- models/ggml-medium.en.bin action_short.wav | tee output.txt
```
