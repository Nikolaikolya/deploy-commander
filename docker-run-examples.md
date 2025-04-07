# Примеры запуска Deploy Commander в Docker

## Базовые команды

### Проверка версии и помощь
```bash
docker run deploy-cmd --version
docker run deploy-cmd --help
```

## Работа с файлами конфигурации

### Монтирование директории с конфигурацией
```bash
docker run -v $(pwd):/workdir deploy-cmd -c /workdir/deploy-config.yml list
```

### Монтирование отдельных файлов конфигурации
```bash
docker run -v $(pwd)/deploy-config.yml:/workdir/config/deploy-config.yml deploy-cmd -c /workdir/config/deploy-config.yml list
```

### Запуск деплоя с монтированием конфигурации
```bash
docker run -v $(pwd):/workdir deploy-cmd -c /workdir/deploy-config.yml run -d my-deployment -e pre-deploy
```

## Использование в CI/CD окружении

### Пример для GitHub Actions
```yaml
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Run deploy command
        run: |
          docker run -v ${{ github.workspace }}:/workdir deploy-cmd -c /workdir/deploy-config.yml run -d production -e deploy
```

### Пример для GitLab CI
```yaml
deploy:
  image: docker:dind
  services:
    - docker:dind
  script:
    - docker run -v $(pwd):/workdir deploy-cmd -c /workdir/deploy-config.yml run -d production -e deploy
```

## Дополнительные примеры

### Запуск с переменными окружения
```bash
docker run -v $(pwd):/workdir -e DB_PASSWORD=secret deploy-cmd -c /workdir/deploy-config.yml run -d dev -e deploy
```

### Монтирование директории логов
```bash
docker run -v $(pwd):/workdir -v $(pwd)/logs:/workdir/logs deploy-cmd -c /workdir/deploy-config.yml run -d staging -e deploy
```

### Пример Makefile для удобного запуска
```makefile
WORKDIR := $(shell pwd)

run-deploy:
	docker run -v $(WORKDIR):/workdir deploy-cmd -c /workdir/deploy-config.yml run -d production -e deploy

verify-config:
	docker run -v $(WORKDIR):/workdir deploy-cmd -c /workdir/deploy-config.yml verify -d production
```

## Особенности нового контейнера

Новая версия контейнера включает улучшенную обработку файлов конфигурации:

1. **Скрипт-обертка** - автоматически проверяет наличие и размер файла конфигурации перед запуском
2. **Расширенное логирование** - выводит подробную информацию о ходе выполнения
3. **Проверка прав доступа** - каталоги имеют разрешения 777 для устранения проблем с правами
4. **Встроенный диагностический инструмент** - доступен через:

```bash
docker run -v $(pwd):/workdir --entrypoint docker-diagnose.sh deploy-cmd
```

## Отладка проблем

Если контейнер не видит файлы, проверьте:

1. Правильно ли смонтированы директории
2. Совпадают ли пути внутри контейнера с путями в командах
3. Имеет ли пользователь в контейнере права на чтение файлов

Для отладки можно использовать встроенный диагностический скрипт:
```bash
docker run -v $(pwd):/workdir --entrypoint docker-diagnose.sh deploy-cmd
```

Или выполнить отдельные команды для проверки:
```bash
# Просмотр содержимого директории
docker run -v $(pwd):/workdir --entrypoint sh deploy-cmd -c "ls -la /workdir && cat /workdir/deploy-config.yml"

# Проверка прав доступа к файлу
docker run -v $(pwd):/workdir --entrypoint sh deploy-cmd -c "stat /workdir/deploy-config.yml"

# Проверка содержимого файла
docker run -v $(pwd):/workdir --entrypoint sh deploy-cmd -c "head -n 20 /workdir/deploy-config.yml"

# Проверка кодировки файла
docker run -v $(pwd):/workdir --entrypoint sh deploy-cmd -c "file /workdir/deploy-config.yml"
```

## Специфичные проблемы и их решения

### Файл не виден внутри контейнера

1. Используйте абсолютные пути при монтировании:
```bash
docker run -v "$(readlink -f ./deploy-config.yml):/workdir/deploy-config.yml" deploy-cmd -c /workdir/deploy-config.yml list
```

2. Проверьте права доступа и владельца файла:
```bash
chmod 644 deploy-config.yml
```

3. Создайте файл непосредственно внутри контейнера:
```bash
docker run -v $(pwd):/workdir --entrypoint sh deploy-cmd -c "cat > /workdir/deploy-config.yml << 'EOF'
deployments:
  - name: test
    description: Test deployment
    events:
      - name: deploy
        commands:
          - command: echo Hello World
EOF
"
```

### Проблемы с правами доступа

В некоторых ситуациях могут возникать проблемы с SELinux. Добавьте флаг `:z` при монтировании:
```bash
docker run -v $(pwd):/workdir:z deploy-cmd -c /workdir/deploy-config.yml list
```

### Проблемы с Windows-путями

При запуске в Windows убедитесь, что пути к файлам указаны в правильном формате:
```powershell
docker run -v ${PWD}:/workdir deploy-cmd -c /workdir/deploy-config.yml list
```

### Проблемы с CRLF

В Windows файлы могут иметь окончания строк CRLF, что иногда приводит к проблемам с YAML-парсером. Убедитесь, что файл использует окончания строк LF:
```powershell
# Преобразование CRLF в LF в PowerShell
(Get-Content -Raw deploy-config.yml) -replace "`r`n", "`n" | Set-Content -NoNewline deploy-config.yml
```

### Проблемы с абсолютными путями в CI/CD окружении

В CI/CD системах (GitLab CI, GitHub Actions) при использовании абсолютных путей возникает проблема, так как внутри контейнера эти пути недоступны:

**Проблема:**
```bash
# НЕ РАБОТАЕТ - путь /builds/... существует на хосте, но не внутри контейнера
docker run --rm deploy-cmd -c /builds/username/project/config/deploy-config.yml run -d my-deploy
```

**Решение 1:** Монтирование с правильным маппингом путей:
```bash
docker run --rm -v /builds/username/project:/workdir/project deploy-cmd -c /workdir/project/config/deploy-config.yml run -d my-deploy
```

**Решение 2:** Для GitLab CI - генерация временной конфигурации в контейнере:
```yaml
deploy:
  script:
    # Копируем файл конфигурации в контейнер
    - docker run --rm -v $CI_PROJECT_DIR:/source --entrypoint sh deploy-cmd -c "mkdir -p /workdir/config && cp /source/config/deploy-config.yml /workdir/config/"
    # Используем скопированный файл
    - docker run --rm -v $CI_PROJECT_DIR:/source deploy-cmd -c /workdir/config/deploy-config.yml run -d bp-app
```

**Решение 3:** Для работы с абсолютными путями используйте специальный скрипт-оболочку:
```bash
#!/bin/bash
# Файл: run-deploy.sh

# Получаем абсолютный путь к файлу конфигурации
CONFIG_PATH="$2"
CONFIG_DIR=$(dirname "$CONFIG_PATH")
CONFIG_FILE=$(basename "$CONFIG_PATH")

# Запускаем контейнер с правильным монтированием
docker run --rm \
  -v "$CONFIG_DIR:/source" \
  deploy-cmd -c "/source/$CONFIG_FILE" "${@:3}"
```

Использование скрипта:
```bash
./run-deploy.sh -c /builds/username/project/config/deploy-config.yml run -d bp-app
``` 