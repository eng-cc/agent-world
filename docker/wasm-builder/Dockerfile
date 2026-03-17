FROM debian:bookworm-slim

ARG WASM_TOOLCHAIN=nightly-2025-12-11
ARG WASM_TARGET=wasm32-unknown-unknown

ENV DEBIAN_FRONTEND=noninteractive
ENV CARGO_HOME=/cargo
ENV RUSTUP_HOME=/rustup
ENV PATH=/cargo/bin:${PATH}
ENV AGENT_WORLD_WASM_TOOLCHAIN=${WASM_TOOLCHAIN}
ENV AGENT_WORLD_WASM_TARGET=${WASM_TARGET}
ENV AGENT_WORLD_WASM_BUILD_STD=1
ENV AGENT_WORLD_WASM_BUILD_STD_COMPONENTS=std,panic_abort
ENV AGENT_WORLD_WASM_BUILD_STD_FEATURES=
ENV AGENT_WORLD_WASM_BUILD_IN_CONTAINER=1
ENV AGENT_WORLD_WASM_BUILD_SUITE_BIN=/cargo/bin/wasm_build_suite
ENV TZ=UTC
ENV LANG=C
ENV LC_ALL=C
ENV SOURCE_DATE_EPOCH=0

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        bash \
        build-essential \
        ca-certificates \
        curl \
        git \
        pkg-config \
        xz-utils \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --default-toolchain none

RUN rustup toolchain install "${WASM_TOOLCHAIN}" --profile minimal --component rust-src \
    && rustup target add "${WASM_TARGET}" --toolchain "${WASM_TOOLCHAIN}"

WORKDIR /opt/agent-world
COPY tools/wasm_build_suite /opt/agent-world/tools/wasm_build_suite
COPY scripts/build-wasm-module.sh /opt/agent-world/scripts/build-wasm-module.sh

RUN chmod +x /opt/agent-world/scripts/build-wasm-module.sh \
    && env -u RUSTC_WRAPPER cargo install --locked --path /opt/agent-world/tools/wasm_build_suite \
    && chmod -R a+rX /cargo /rustup /opt/agent-world

LABEL org.opencontainers.image.title="agent-world wasm builder"
LABEL org.opencontainers.image.description="Canonical linux-x86_64 WASM builder for Agent World"
LABEL org.opencontainers.image.source="https://example.invalid/agent-world"
LABEL org.opencontainers.image.vendor="Agent World"

WORKDIR /workspace
ENTRYPOINT ["/opt/agent-world/scripts/build-wasm-module.sh"]
