FROM rust:1.80-slim as builder

WORKDIR /usr/src/deploy-commander
COPY . .

RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    cargo build --release

# Финальный контейнер
FROM ubuntu:minimal

# Устанавливаем только необходимые зависимости
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      ca-certificates \
      libssl-dev \
      git \
      openssh-client \
      rsync && \
    rm -rf /var/lib/apt/lists/*

# Копируем бинарник
COPY --from=builder /usr/src/deploy-commander/target/release/deploy-cmd /usr/local/bin/deploy-cmd

# Настройка окружения
RUN mkdir -p /app/config

WORKDIR /app

COPY ./examples/deploy-config.yml /app/config/deploy-config.yml

ENTRYPOINT ["deploy-cmd", "-c", "/app/config/deploy-config.yml"]
CMD ["--help"]