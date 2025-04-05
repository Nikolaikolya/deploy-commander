# Deploy Commander

Deploy Commander - это инструмент для управления выполнением команд при деплое на сервере.
Он позволяет структурировать процесс деплоя, автоматизировать выполнение команд и отслеживать их выполнение.

## Возможности

- Управление командами деплоя через YAML конфигурацию
- Группировка команд по событиям (pre-deploy, deploy, post-deploy и др.)
- Контроль времени выполнения команд с таймаутами
- Гибкая настройка рабочих директорий и переменных окружения
- Отслеживание истории деплоев
- Уведомления о результатах выполнения

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

```bash
deploy-cmd run -d my-app -e deploy
```

### Просмотр доступных деплоев

```bash
deploy-cmd list
```

### Проверка конфигурации

```bash
deploy-cmd verify -d my-app
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
      - name: deploy
        description: "Основной процесс деплоя"
        commands:
          - command: "git pull origin main"
            description: "Получение изменений"
            timeout: 60
          - command: "npm ci"
            description: "Установка зависимостей"
            timeout: 300
        fail_fast: true
```

## Интеграция с CI/CD

Deploy Commander можно интегрировать с различными системами CI/CD:

### GitLab CI

```yaml
deploy:
  stage: deploy
  script:
    - deploy-cmd run -d my-app -e deploy
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
        run: deploy-cmd run -d my-app -e deploy
```

## Лицензия

MIT