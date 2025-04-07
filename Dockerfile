# Используем официальный минимальный образ Ubuntu
FROM ubuntu:20.04

# Обновляем пакеты и устанавливаем curl
RUN apt-get update && \
  apt-get install -y curl && \
  # Скачиваем бинарник deploy-cmd и делаем его исполнимым
  curl -L https://github.com/Nikolaikolya/deploy-commander/releases/download/v1.0.3/deploy-cmd -o /usr/local/bin/deploy-cmd && \
  chmod +x /usr/local/bin/deploy-cmd

# Устанавливаем deploy-cmd как основной исполнимый файл
ENTRYPOINT ["deploy-cmd"]

# Устанавливаем аргумент по умолчанию
CMD ["--help"]