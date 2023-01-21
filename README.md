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

Complete set of tools for making your favourite podcast searchable.

Originally created for [jupiter network](https://www.jupiterbroadcasting.com/) podcasts using [meilisearch](https://www.meilisearch.com/).

- [search issue](https://github.com/JupiterBroadcasting/jupiterbroadcasting.com/issues/26)
- [transcription issue](https://github.com/JupiterBroadcasting/jupiterbroadcasting.com/issues/301)

## Overview

Project contains two main modules:

* `podcast2text` a cli tool for downloading RSS feed and transcribing podcast episodes 
* `search-load` a cli tool for loading obtained transcriptions to
  instance of meilisearch


## Getting started

To build you would need following packages on your system:

- cargo
- pkg-config
- openssl
- ffmpeg

There is a nix flake configured to ship build dependencies
just run `direnv allow` and run:

```shell
cargo build --release
```

To appease the gods of good taste please add following pre commit hook:

```
git config --local core.hooksPath .githooks
```

## Usage

### Run downloading podcasts

#### Process audio from RSS feed


1. Create cache directories and download the whisper model

```shell
mkdir -p {models,output}
# this might be one of:
# "tiny.en" "tiny" "base.en" "base" "small.en" "small" "medium.en" "medium" "large"
model=tiny.en

curl -L --output models/$model.bin https://huggingface.co/datasets/ggerganov/whisper.cpp/resolve/main/ggml-$model.bin
```

2. Run the inference on the RSS feed

```shell
# get information about the cli
docker run flakm/podcast2text rss --help

docker run \
    -v $PWD/models:/data/models \
    -v $PWD/output:/data/output \
    flakm/podcast2text \
    rss \
    --num-of-episodes 2 \
    https://feed.jupiter.zone/allshows 

# or using cargo
cargo run --bin podcast2text --release -- \
    --model-path=models/model.bin \
    --output-dir=output/ \
    --download-dir=catalog \
    rss \
    --num-of-episodes 1 \
    https://feed.jupiter.zone/allshows 
```

The output directory should now contain json files with files'
transcription and metadata. Note that the results will be cached - so if
you restart the job it will not redownload and process already seen
rss entries.


### Create search engine

#### Install meilisearch

Project uses [meilisearch](https://www.meilisearch.com/) as engine
back end for search functionality

```shell
docker pull getmeili/meilisearch:v0.29
docker run -it --rm \
    -p 7700:7700 \
    -e MEILI_MASTER_KEY='MASTER_KEY'\
    -v $(pwd)/meili_data:/meili_data \
    getmeili/meilisearch:v0.29 \
    meilisearch --env="development"
```

#### Run index creation and data loading

