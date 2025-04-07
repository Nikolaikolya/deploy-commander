# Используем более новый образ Ubuntu с более новой версией GLIBC
FROM ubuntu:24.04

# Обновляем пакеты и устанавливаем необходимые инструменты
RUN apt-get update && \
  apt-get install -y curl git ssh docker.io rsync file && \
  apt-get clean && \
  rm -rf /var/lib/apt/lists/*

# Скачиваем бинарник deploy-cmd и делаем его исполнимым
RUN curl -L https://github.com/Nikolaikolya/deploy-commander/releases/download/v1.0.3/deploy-cmd -o /usr/local/bin/deploy-cmd && \
  chmod +x /usr/local/bin/deploy-cmd

# Создаем рабочую директорию для проекта
WORKDIR /workdir

# Создаем директории для конфигурации и логов
RUN mkdir -p /workdir/config && \
  mkdir -p /workdir/logs && \
  chmod 777 /workdir /workdir/config /workdir/logs

# Создаем диагностический скрипт
RUN echo '#!/bin/bash\n\
  echo "==== Информация о системе ===="\n\
  echo "Текущая директория: $(pwd)"\n\
  echo "Содержимое /workdir:"\n\
  ls -la /workdir\n\
  echo ""\n\
  \n\
  echo "==== Проверка config файла ===="\n\
  if [ -f "/workdir/deploy-config.yml" ]; then\n\
  echo "Файл /workdir/deploy-config.yml существует"\n\
  echo "Размер: $(stat -c%s /workdir/deploy-config.yml) байт"\n\
  echo "Права: $(stat -c%a /workdir/deploy-config.yml)"\n\
  echo "Владелец: $(stat -c%U /workdir/deploy-config.yml)"\n\
  echo "Первые 10 строк файла:"\n\
  head -10 /workdir/deploy-config.yml\n\
  else\n\
  echo "Файл /workdir/deploy-config.yml НЕ существует"\n\
  fi\n\
  echo ""\n\
  \n\
  echo "==== Проверка монтирования ===="\n\
  echo "mount | grep workdir:"\n\
  mount | grep workdir\n\
  echo ""\n\
  \n\
  echo "==== Проверка символических ссылок ===="\n\
  find /workdir -type l -ls\n\
  echo ""\n\
  \n\
  echo "==== Информация о версии deploy-cmd ===="\n\
  deploy-cmd --version\n\
  echo ""\n\
  \n\
  echo "Диагностика завершена."' > /usr/local/bin/docker-diagnose.sh && \
  chmod +x /usr/local/bin/docker-diagnose.sh

# Создаем скрипт-обертку для лучшей обработки файлов конфигурации
RUN echo '#!/bin/bash\n\
  \n\
  # Проверяем наличие файла конфигурации\n\
  CONFIG_FILE="$2"\n\
  \n\
  if [ "$1" = "-c" ] && [ -n "$CONFIG_FILE" ]; then\n\
  echo "Проверка файла конфигурации: $CONFIG_FILE"\n\
  \n\
  if [ ! -f "$CONFIG_FILE" ]; then\n\
  echo "ОШИБКА: Файл конфигурации не найден: $CONFIG_FILE"\n\
  echo "Текущая директория: $(pwd)"\n\
  echo "Содержимое $(dirname $CONFIG_FILE):"\n\
  ls -la $(dirname $CONFIG_FILE)\n\
  exit 1\n\
  fi\n\
  \n\
  # Проверка содержимого файла\n\
  if [ ! -s "$CONFIG_FILE" ]; then\n\
  echo "ОШИБКА: Файл конфигурации пустой: $CONFIG_FILE"\n\
  exit 1\n\
  fi\n\
  \n\
  echo "Файл конфигурации найден, размер: $(stat -c%s $CONFIG_FILE) байт"\n\
  echo "Первые 5 строк файла:"\n\
  head -n 5 "$CONFIG_FILE"\n\
  echo "..."\n\
  fi\n\
  \n\
  # Запускаем оригинальную команду\n\
  deploy-cmd "$@"' > /usr/local/bin/deploy-wrapper.sh && \
  chmod +x /usr/local/bin/deploy-wrapper.sh

# Устанавливаем deploy-wrapper.sh как основной исполнимый файл
ENTRYPOINT ["/usr/local/bin/deploy-wrapper.sh"]

# Устанавливаем аргумент по умолчанию
CMD ["--help"]