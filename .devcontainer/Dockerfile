FROM debian:testing@sha256:251c723aad0e93dbd71d566c2341dcbc0767d67b0022a9ca3f0e1f99f6da355f

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
