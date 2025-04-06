# Примеры использования Deploy Commander

В данной директории содержатся примеры файлов конфигурации и команд для использования Deploy Commander.

## Примеры конфигурации

### `deploy-config.yml`

Основной файл конфигурации с реальными путями и командами для развертывания приложений в рабочей среде.

### `local-deploy-config.yml`

Конфигурация для локального тестирования с относительными путями и временными директориями. Готова к использованию без необходимости настройки каких-либо путей в системе.

## Функции системы команд

- **Выполнение цепочки команд**: Последовательное выполнение команд в рамках одного события
- **Автоматическое логирование**: Вывод информации о выполнении команд
- **Отображение ошибок**: Вывод детальной информации об ошибках
- **Создание директорий**: Автоматическое создание рабочих директорий
- **Обработка ошибок и откат**: Система может обрабатывать ошибки и выполнять откат изменений

## Параметры обработки ошибок и отката

- **ignore_errors**: Если установлено значение `true`, исполнение продолжится даже при ошибке в команде
- **rollback_command**: Команда, которая будет выполнена для отката изменений, если основная команда завершится с ошибкой
- **fail_fast**: Если установлено значение `true`, цепочка команд остановится при первой ошибке

## Примеры команд

### Использование локальной конфигурации

```bash
# Инициализация тестовой среды
cargo run -- -c ./examples/local-deploy-config.yml run -d get-all-projects

# Деплой фронтенд приложения
cargo run -- -c ./examples/local-deploy-config.yml run -d frontend-app

# Деплой бэкенд API
cargo run -- -c ./examples/local-deploy-config.yml run -d backend-api

# Тестирование функции отката
cargo run -- -c ./examples/local-deploy-config.yml run -d backend-api -e rollback-test
```

### Проверка конфигурации

```bash
# Вывести список всех команд в конфигурации
cargo run -- -c ./examples/local-deploy-config.yml list-commands

# Проверить конфигурацию на ошибки
cargo run -- -c ./examples/local-deploy-config.yml verify
```

### Просмотр истории деплоя

```bash
# Показать историю деплоя
cargo run -- -c ./examples/local-deploy-config.yml history
```

## Примеры реализации отката

### Откат при ошибке в команде

```yaml
- command: "mkdir -p ./dist"
  description: "Создание директории для дистрибутива"
  ignore_errors: false
  rollback_command: "rm -rf ./dist"
```

### Откат для команд Docker

```yaml
- command: "docker build -t my-app ."
  description: "Сборка Docker-образа"
  ignore_errors: false
  rollback_command: "docker rmi my-app || true"
```

### Откат для Git-команд

```yaml
- command: "git pull origin main"
  description: "Обновление репозитория"
  ignore_errors: false
  rollback_command: "git reset --hard HEAD~1"
```

## Особенности использования Command System

Deploy Commander использует библиотеку Command System для эффективного выполнения команд:

1. Команды собираются в цепочки и выполняются последовательно
2. Результаты команд автоматически логируются
3. Сообщения об ошибках и выводы команд отображаются в консоли

## Примеры команд

### Запуск всех событий для деплоя frontend-app

```bash
cargo run -- -c ./examples/deploy-config.yml run -d frontend-app
```

### Запуск конкретного события для деплоя frontend-app

```bash
cargo run -- -c ./examples/deploy-config.yml run -d frontend-app -e deploy
```

### Запуск всех событий для деплоя backend-api

```bash
cargo run -- -c ./examples/deploy-config.yml run -d backend-api
```

### Просмотр списка доступных деплоев и событий

```bash
cargo run -- -c ./examples/deploy-config.yml list
```

### Проверка конфигурации деплоя

```bash
cargo run -- -c ./examples/deploy-config.yml verify -d frontend-app
```

### Просмотр истории деплоев

```bash
cargo run -- -c ./examples/deploy-config.yml history -d frontend-app -l 5
```

### Очистка истории деплоев

```bash
cargo run -- -c ./examples/deploy-config.yml clear-history -d frontend-app
```

## Порядок выполнения событий

При запуске деплоя без указания конкретного события (например, `run -d frontend-app`), события будут выполнены в том порядке, в котором они определены в конфигурации.

Выполнение команд внутри одного события происходит последовательно, с использованием цепочки Command System. Если одна из команд завершается с ошибкой и для события установлен флаг fail_fast: true, выполнение текущего события останавливается.

Если при выполнении одного из событий происходит ошибка, деплой останавливается и последующие события не выполняются. 