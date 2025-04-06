# Deploy Commander

Deploy Commander - это инструмент для управления выполнением команд при деплое на сервере.
Он позволяет структурировать процесс деплоя, автоматизировать выполнение команд и отслеживать их выполнение.

## Возможности

- Управление командами деплоя через YAML конфигурацию
- Группировка команд по событиям (pre-deploy, deploy, post-deploy и др.)
- Последовательное выполнение команд с выводом результатов в консоль
- Гибкая настройка рабочих директорий и переменных окружения
- Отслеживание и хранение истории деплоев
- Возможность запуска как отдельных событий, так и полного цикла деплоя
- Управление историей деплоев (просмотр, очистка)

## Установка

```bash
cargo install deploy-commander
```

Или сборка из исходного кода:

```bash
git clone https://github.com/yourusername/deploy-commander.git
cd deploy-commander
cargo build --release
```

## Использование

### Создание шаблона конфигурации

```bash
deploy-cmd create -d my-app
```

### Запуск деплоя

#### Запуск конкретного события

```bash
deploy-cmd run -d my-app -e deploy
```

#### Запуск всех событий последовательно

```bash
deploy-cmd run -d my-app
```

#### Запуск с указанием пути к конфигурации

```bash
deploy-cmd -c ./examples/deploy-config.yml run -d frontend-app
```

### Просмотр доступных деплоев

```bash
deploy-cmd list
```

### Проверка конфигурации

```bash
deploy-cmd verify -d my-app
```

### Работа с историей деплоев

#### Просмотр истории деплоя

```bash
deploy-cmd history -d my-app -l 20
```

#### Очистка истории для конкретного деплоя

```bash
deploy-cmd clear-history -d my-app
```

#### Очистка всей истории

```bash
deploy-cmd clear-history
```

## Структура конфигурации

Конфигурация хранится в формате YAML и содержит описание деплоев, событий и команд:

```yaml
deployments:
  - name: my-app
    description: "Деплой моего приложения"
    working_dir: "/var/www/my-app"
    environment:
      - "NODE_ENV=production"
    events:
      - name: pre-deploy
        description: "Подготовка к деплою"
        commands:
          - command: "echo 'Начало деплоя'"
            description: "Вывод сообщения о начале деплоя"
            ignore_errors: true
        fail_fast: true
      - name: deploy
        description: "Основной процесс деплоя"
        commands:
          - command: "git pull origin main"
            description: "Получение изменений"
          - command: "npm ci"
            description: "Установка зависимостей"
          - command: "npm run build"
            description: "Сборка проекта"
        fail_fast: true
      - name: post-deploy
        description: "Действия после деплоя"
        commands:
          - command: "pm2 restart app"
            description: "Перезапуск приложения"
          - command: "echo 'Деплой завершен'"
            description: "Вывод сообщения о завершении деплоя"
            ignore_errors: true
        fail_fast: false
```

## Описание параметров конфигурации

- `name` - имя деплоя или события
- `description` - описание (опционально)
- `working_dir` - рабочая директория для выполнения команд (опционально)
- `environment` - список переменных окружения в формате KEY=VALUE (опционально)
- `events` - список событий деплоя
- `commands` - список команд для выполнения
- `ignore_errors` - игнорировать ошибки выполнения команды (опционально, по умолчанию false)
- `fail_fast` - остановить выполнение событий при первой ошибке (опционально, по умолчанию true)

## Интеграция с CI/CD

Deploy Commander можно интегрировать с различными системами CI/CD:

### GitLab CI

```yaml
deploy:
  stage: deploy
  script:
    - deploy-cmd run -d my-app
  only:
    - main
```

### GitHub Actions

```yaml
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Deploy application
        run: deploy-cmd run -d my-app
```

## Лицензия

MIT