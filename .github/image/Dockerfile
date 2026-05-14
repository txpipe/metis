FROM debian:12-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && curl -fsSL https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash \
    && apt-get purge -y --auto-remove curl \
    && rm -rf /var/lib/apt/lists/*

ARG TARGETARCH
ARG BIN
COPY bin/${BIN}-Linux-${TARGETARCH} /bin/entrypoint
RUN chmod +x /bin/entrypoint

ENTRYPOINT [ "/bin/entrypoint" ]
