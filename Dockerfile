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

# Создаем директории для монтирования из CI систем
RUN mkdir -p /builds && chmod 777 /builds

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
  # Функция для получения реального пути к файлу конфигурации\n\
  handle_config_path() {\n\
  local config_path="$1"\n\
  \n\
  # Проверяем, существует ли файл напрямую\n\
  if [ -f "$config_path" ]; then\n\
  echo "$config_path"\n\
  return 0\n\
  fi\n\
  \n\
  # Проверяем абсолютный путь от корня хоста (CI/CD системы)\n\
  if [[ "$config_path" == /builds/* ]]; then\n\
  # Путь в формате GitLab CI\n\
  local path_in_container="/workdir/$(basename $config_path)"\n\
  echo "CI/CD путь обнаружен: $config_path"\n\
  echo "Попытка копирования файла внутрь контейнера..."\n\
  \n\
  # Проверяем, смонтирована ли директория /builds напрямую\n\
  if [ -d "$(dirname $config_path)" ]; then\n\
  echo "Директория видна в контейнере, копируем файл..."\n\
  mkdir -p "$(dirname $path_in_container)"\n\
  cp "$config_path" "$path_in_container"\n\
  echo "Файл скопирован в: $path_in_container"\n\
  echo "$path_in_container"\n\
  return 0\n\
  fi\n\
  \n\
  # Проверяем, смонтирована ли директория через /workdir\n\
  local source_base_dir=$(echo "$config_path" | sed "s|^/builds/||")\n\
  if [ -f "/workdir/$source_base_dir" ]; then\n\
  echo "Файл найден через /workdir: /workdir/$source_base_dir"\n\
  echo "/workdir/$source_base_dir"\n\
  return 0\n\
  fi\n\
  \n\
  # Проверяем различные варианты путей\n\
  local project_dir=$(echo "$config_path" | grep -o "/builds/[^/]*/[^/]*" | head -1)\n\
  if [ -n "$project_dir" ] && [ -d "/workdir" ]; then\n\
  local relative_path=$(echo "$config_path" | sed "s|^$project_dir/||")\n\
  if [ -f "/workdir/$relative_path" ]; then\n\
  echo "Файл найден по относительному пути: /workdir/$relative_path"\n\
  echo "/workdir/$relative_path"\n\
  return 0\n\
  fi\n\
  fi\n\
  fi\n\
  \n\
  # Файл не найден\n\
  echo "$config_path"\n\
  return 1\n\
  }\n\
  \n\
  # Проверяем наличие флага конфигурации\n\
  if [ "$1" = "-c" ] && [ -n "$2" ]; then\n\
  CONFIG_FILE="$2"\n\
  echo "Проверка файла конфигурации: $CONFIG_FILE"\n\
  \n\
  # Преобразуем путь, если необходимо\n\
  RESOLVED_CONFIG=$(handle_config_path "$CONFIG_FILE")\n\
  CONFIG_RESULT=$?\n\
  \n\
  if [ $CONFIG_RESULT -ne 0 ]; then\n\
  # Путь не удалось преобразовать и файл не существует\n\
  echo "ОШИБКА: Файл конфигурации не найден: $CONFIG_FILE"\n\
  echo "Текущая директория: $(pwd)"\n\
  echo "Содержимое $(dirname $CONFIG_FILE 2>/dev/null || echo "."):\n\
  if [ -d "$(dirname $CONFIG_FILE 2>/dev/null)" ]; then\n\
  ls -la "$(dirname $CONFIG_FILE)"\n\
  else\n\
  echo "Директория не существует или недоступна"\n\
  echo "Содержимое /workdir:"\n\
  ls -la /workdir\n\
  fi\n\
  exit 1\n\
  fi\n\
  \n\
  # Проверка содержимого файла\n\
  if [ ! -s "$RESOLVED_CONFIG" ]; then\n\
  echo "ОШИБКА: Файл конфигурации пустой: $RESOLVED_CONFIG"\n\
  exit 1\n\
  fi\n\
  \n\
  echo "Файл конфигурации найден, размер: $(stat -c%s $RESOLVED_CONFIG) байт"\n\
  echo "Первые 5 строк файла:"\n\
  head -n 5 "$RESOLVED_CONFIG"\n\
  echo "..."\n\
  \n\
  # Заменяем аргумент с путем на новый преобразованный путь\n\
  shift 2\n\
  set -- "-c" "$RESOLVED_CONFIG" "$@"\n\
  fi\n\
  \n\
  # Запускаем оригинальную команду\n\
  deploy-cmd "$@"' > /usr/local/bin/deploy-wrapper.sh && \
  chmod +x /usr/local/bin/deploy-wrapper.sh

# Устанавливаем deploy-wrapper.sh как основной исполнимый файл
ENTRYPOINT ["/usr/local/bin/deploy-wrapper.sh"]

# Устанавливаем аргумент по умолчанию
CMD ["--help"]