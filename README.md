# Jupiter Search

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[![APACHE 2 licensed][apache-badge]][apache-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/podcast2text.svg
[crates-url]: https://crates.io/crates/podcast2text
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/FlakM/jupiter-search/blob/master/LICENSE-MIT
[apache-badge]: https://img.shields.io/badge/License-Apache_2.0-blue.svg
[apache-url]: https://github.com/FlakM/jupiter-search/blob/master/LICENSE-APACHE
[actions-badge]: https://github.com/flakm/jupiter-search/actions/workflows/build.yml/badge.svg
[actions-url]: https://github.com/FlakM/jupiter-search/actions


A showcase for indexing [jupiter network](https://www.jupiterbroadcasting.com/) podcasts using [meilisearch](https://www.meilisearch.com/).
This repository is build in order to provide possible solution to following problems:

- [search](https://github.com/JupiterBroadcasting/jupiterbroadcasting.com/issues/26)
- [transcription](https://github.com/JupiterBroadcasting/jupiterbroadcasting.com/issues/301)

**DISCLAIMER!**

Warning! This is a work in progress version to showcase how indexing/transcription might work.

## Overview

Project contains two main modules:

* `podcast2text` a cli tool for downloading RSS feed and transcribing podcast episodes 
* `search-load` a cli tool for loading obtained transcriptions to
  instance of meilisearch


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
cargo build --release
```

To appease the gods of good taste please add following pre commit hook:

```
git config --local core.hooksPath .githooks
```

## Usage

### Run downloading podcasts

### Process audio from RSS feed


1. Download the whisper model

```shell
mkdir models
# this might be one of:
# "tiny.en" "tiny" "base.en" "base" "small.en" "small" "medium.en" "medium" "large"
model=medium.en
curl --output models/model.bin https://ggml.ggerganov.com/ggml-model-whisper-$model.bin
```

2. Run the inference on the RSS feed

```shell
# get information about the cli
docker run flakm/podcast2text --help

docker run \
    -v $PWD/models:/data/models \
    flakm/podcast2text \
    rss https://feed.jupiter.zone/allshows
```




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
