FROM mimirbridge/hc-proxy-base

WORKDIR /logs

COPY config.toml config.toml

COPY templates templates

WORKDIR /

ENTRYPOINT ["eth-proxy", "stream-logs", "--config-file", "logs/config.toml", "--template-dir", "logs/templates"]

CMD ["--node-addr","ws://10.0.10.2:8546"]

