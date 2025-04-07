#!/bin/bash

# Скрипт для сборки Docker-образа deploy-cmd

# Версия приложения
VERSION=$(grep version Cargo.toml | head -1 | awk -F'"' '{print $2}')
DOCKER_IMAGE="deploy-cmd:${VERSION}"

echo "Сборка образа ${DOCKER_IMAGE}..."

# Сборка Docker-образа
docker build -t ${DOCKER_IMAGE} -f Dockerfile .

# Вывод инструкций по использованию
echo -e "\nОбраз успешно собран: ${DOCKER_IMAGE}"
echo -e "\nПримеры использования:"
echo -e "  Базовая проверка:"
echo -e "    docker run ${DOCKER_IMAGE} --version"
echo -e "    docker run ${DOCKER_IMAGE} --help"
echo -e "\n  Запуск с файлом конфигурации:"
echo -e "    docker run -v \$(pwd):/workdir ${DOCKER_IMAGE} -c /workdir/deploy-config.yml list"
echo -e "\n  Запуск деплоя:"
echo -e "    docker run -v \$(pwd):/workdir ${DOCKER_IMAGE} -c /workdir/deploy-config.yml run -d your-deployment -e your-event"
echo -e "\n  Диагностика проблем с файлами:"
echo -e "    docker run -v \$(pwd):/workdir --entrypoint docker-diagnose.sh ${DOCKER_IMAGE}"
echo -e "\n  Для подробных примеров см. docker-run-examples.md"

# Информация о новой функциональности
echo -e "\nНовые возможности в этой версии контейнера:"
echo -e "  * Встроенная проверка доступности файлов конфигурации"
echo -e "  * Расширенная диагностика с помощью docker-diagnose.sh"
echo -e "  * Исправлены проблемы с доступом к файлам"
echo -e "  * Скрипт-обертка для обнаружения ошибок конфигурации"

# Создание тега latest
docker tag ${DOCKER_IMAGE} deploy-cmd:latest
echo -e "\nСоздан дополнительный тег: deploy-cmd:latest" 