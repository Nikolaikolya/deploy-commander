FROM rust:1.70-slim as builder

WORKDIR /usr/src/deploy-commander
COPY . .

RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    cargo build --release

FROM debian:bullseye-slim

RUN apt-get update && \
    apt-get install -y ca-certificates libssl-dev git openssh-client rsync && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/deploy-commander/target/release/deploy-cmd /usr/local/bin/deploy-cmd

# Создаем директорию для конфигурации
RUN mkdir -p /app/config

WORKDIR /app

# Устанавливаем конфигурацию по умолчанию
COPY ./examples/deploy-config.yml /app/config/deploy-config.yml

ENTRYPOINT ["deploy-cmd", "-c", "/app/config/deploy-config.yml"]
CMD ["--help"]