FROM rust:latest

COPY eth-proxy eth-proxy

WORKDIR /eth-proxy

RUN cargo install --path .

WORKDIR /

RUN rm -r eth-proxy

ENTRYPOINT ["eth-proxy"]

