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

`cargo run`


### Running inference of some audio

1. Download the models from
   [coqui](https://coqui.ai/english/coqui/v1.0.0-large-vocab) you will
   need acoustic model and scorer.

```
mkdir models
# this might be one of:
#  "tiny.en" "tiny" "base.en" "base" "small.en" "small" "medium.en" "medium" "large"
model=medium.en
curl --output ggml-$model.bin https://ggml.ggerganov.com/ggml-model-whisper-$model.bin
```
2. Download the audio from rss feed

```
curl https://feed.jupiter.zone/link/19057/15745245/55bb5263-04be-43a3-8b92-678072a9cfc8.mp3 -L -o action.mp3
```

3. Convert it to 16kHz, mono-channel audio (60 seconds for testing
   purposes) 

```
ffmpeg -i action.mp3 -ar 16000 -T 60 action_short.wav
```

4. Run the inference example

```
cd whisper.cpp
make
cd -
cargo run --release --example=get_transcript -- models/ggml-medium.en.bin action_short.wav | tee output.txt
    Finished release [optimized] target(s) in 0.10s
     Running `target/release/examples/get_transcript models action_short.wav`
TensorFlow: v2.3.0-6-g23ad988fcde
 Coqui STT: v0.10.0-alpha.4-74-g49cdf7a6
2022-11-06 22:08:05.103566: I tensorflow/core/platform/cpu_feature_guard.cc:142] This TensorFlow binary is optimized with oneAPI Deep Neural Network Library (oneDNN)to use the following CPU instructions in performance-critical operations:  AVX2 FMA
To enable them in other operations, rebuild TensorFlow with the appropriate compiler flags.
Using external scorer `models/model.scorer`
0ns - 8.7s [8.7s] -> kueelo and welcome to limitation news episode two hundred and sixty four record about october twenty six
8.78s - 10.42s [1.64s] -> i'm chris
10.66s - 12.56s [1.9s] -> i'm less hello was
12.72s - 13.74s [1.02s] -> so the news
14.5s - 18.04s [3.54s] -> an instant twenty two ten a week
18.4s - 24.74s [6.34s] -> god named connecticut this interim release is to have a particular focus on the rabbit pie
...
took 61.825862705s
```
