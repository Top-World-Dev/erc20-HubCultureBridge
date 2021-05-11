FROM mimirbridge/basil-client:latest

WORKDIR /

COPY ./ ./

WORKDIR /basil

VOLUME /basil/state

RUN chmod +x init.sh && ./init.sh

WORKDIR /

CMD ["--config","basil/config.toml","--jsonrpc-cors","all","--jsonrpc-hosts","all"]

