FROM mimirbridge/hc-proxy-base

WORKDIR /signer

COPY ./ ./

WORKDIR /

VOLUME /signer

ENTRYPOINT ["eth-proxy","signer-proxy","0.0.0.0:8080","--signer-config","signer/config.toml","--secret-cache","signer/secret.key"]

