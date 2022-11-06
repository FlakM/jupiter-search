# jupiter-search

Showcase for indexing jupiter network podcasts using meilisearch.
This repository is build in order to provide possible solution to following problems:

- [search](https://github.com/JupiterBroadcasting/jupiterbroadcasting.com/issues/26)
- [transcription](https://github.com/JupiterBroadcasting/jupiterbroadcasting.com/issues/301)

**DISCLAIMER!**

Warning! This is a very dirty version to showcase how indexing/transcription might work.


## Building


To build you would need following packages on your system:

- stt
- cargo
- pkg-config
- openssl

There is a nix flake configured to ship build dependencies
just run `direnv allow` and run:

```shell
cargo build
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

