FROM rust:1.33.0-stretch as build
RUN apt update && apt install -y \
        curl \
        build-essential \
    && rm -rf /var/lib/apt/lists/* && \
    mkdir -p /root/.cargo && \
    ln -sf /usr/bin /root/.cargo
WORKDIR /workdir
ADD ./pm /workdir
RUN cargo build --release

FROM debian:stretch as release
RUN apt update && apt install -y xorg-dev \
    && rm -rf /var/lib/apt/lists/* && \
    mkdir -p /root/.cargo && \
    ln -sf /usr/bin /root/.cargo
COPY --from=build /workdir/target/release/pm /bin/pm
WORKDIR /workdir
