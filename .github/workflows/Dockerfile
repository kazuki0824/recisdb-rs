ARG IMAGE
FROM ${IMAGE}
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        clang \
        cmake \
        curl \
        libpcsclite-dev \
        libudev-dev \
        libdvbv5-dev \
        pkg-config && \
    rm -rf /var/lib/apt/lists/*
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain stable -y && \
    /root/.cargo/bin/cargo install cargo-deb --version 2.11.2 --locked
COPY . /src/
WORKDIR /src/
