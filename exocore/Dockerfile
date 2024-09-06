FROM rust:slim

RUN apt-get update && \
    apt-get install -y build-essential pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

COPY . /opt/exocore/

RUN cd /opt/exocore && \
    ./tools/install.sh && \
    rm -rf /opt/exocore/ && \
    rm -rf /usr/local/cargo/registry/

WORKDIR /volume
ENTRYPOINT ["/usr/local/cargo/bin/exo"]
