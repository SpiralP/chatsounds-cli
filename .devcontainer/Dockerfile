FROM debian:testing@sha256:8ac1a6436385cbc6aee56b98a2671c26a26d1f74d6da42d58084c2179355edde

RUN export DEBIAN_FRONTEND=noninteractive \
  && apt-get -y update \
  && apt-get -y install --no-install-recommends \
  # common
  curl ca-certificates git gcc g++ build-essential pkg-config \
  # openssl-sys <- reqwest
  libssl-dev \
  # alsa-sys
  libasound2-dev \
  && curl -sS 'https://sh.rustup.rs' | sh -s -- -y --default-toolchain none \
  && ~/.cargo/bin/rustup default nightly \
  && ~/.cargo/bin/rustup component add rust-src rustfmt clippy
