# Этап сборки
FROM rust:1.86 AS builder

WORKDIR /app
COPY . .

RUN cargo build --release

# Финальный образ
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Копируем бинарник и конфиг
COPY --from=builder /app/target/release/deploy-cmd .
COPY settings.json .

RUN ls -la /app

# Создаём папку для логов (как указано в settings.json)
RUN mkdir -p /app/logs

## Устанавливаем deploy-wrapper.sh как основной исполнимый файл
ENTRYPOINT ["/app/deploy-cmd"]
#
## Устанавливаем аргумент по умолчанию
CMD ["--help"]



## Используем более новый образ Ubuntu с более новой версией GLIBC
#FROM ubuntu:24.04
#
## Обновляем пакеты и устанавливаем необходимые инструменты
#RUN apt-get update && \
#  apt-get install -y curl git ssh docker.io rsync file && \
#  apt-get clean && \
#  rm -rf /var/lib/apt/lists/*
#
## Скачиваем бинарник deploy-cmd и делаем его исполнимым
#RUN curl -L https://github.com/Nikolaikolya/deploy-commander/releases/download/v1.0.3/deploy-cmd -o /usr/local/bin/deploy-cmd && \
#  chmod +x /usr/local/bin/deploy-cmd
#
## Создаем рабочую директорию для проекта
#WORKDIR /workdir
#
## Создаем директории для конфигурации и логов
#RUN mkdir -p /workdir/config && \
#  mkdir -p /workdir/logs && \
#  chmod 777 /workdir /workdir/config /workdir/logs
#
## Создаем директории для монтирования из CI систем
#RUN mkdir -p /builds && chmod 777 /builds
#
## Устанавливаем deploy-wrapper.sh как основной исполнимый файл
#ENTRYPOINT ["/usr/local/bin/deploy-wrapper.sh"]
#
## Устанавливаем аргумент по умолчанию
#CMD ["--help"]