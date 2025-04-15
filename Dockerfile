# Этап сборки
FROM rust:1.86 AS builder

WORKDIR /app
COPY . .

RUN cargo build --release

# Финальный образ
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y ca-certificates curl gnupg && \
    mkdir -p /etc/apt/keyrings && \
    curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg && \
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian bookworm stable" > /etc/apt/sources.list.d/docker.list && \
    apt-get update && \
    apt-get install -y docker-ce-cli && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Копируем бинарник и конфиг
COPY --from=builder /app/target/release/deploy-cmd .
COPY settings.json .

RUN ls -la /app

# Создаём папку для логов (как указано в settings.json)
RUN mkdir -p /app/log

## Устанавливаем deploy-wrapper.sh как основной исполнимый файл
ENTRYPOINT ["/app/deploy-cmd"]

## Устанавливаем аргумент по умолчанию
CMD ["--help"]