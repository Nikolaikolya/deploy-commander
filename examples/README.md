# Примеры использования Deploy Commander

В данной директории содержатся примеры файлов конфигурации и команд для использования Deploy Commander.

## Примеры конфигурации

### `deploy-config.yml`

Основной файл конфигурации с реальными путями и командами для развертывания приложений в рабочей среде.

### `local-deploy-config.yml`

Конфигурация для локального тестирования с относительными путями и временными директориями. Готова к использованию без необходимости настройки каких-либо путей в системе.

### `interactive-config-example.yml`

Пример конфигурации для демонстрации работы с переменными, включая интерактивный ввод, переменные из файлов (JSON и YAML) и переменные окружения.

## Примеры тестирования переменных

### `direct_variables_file_test.rs`

Пример прямого тестирования загрузки переменных из JSON и YAML файлов с помощью CommandBuilder:
```bash
cargo run --example direct_variables_file_test
```

### `variable_demo.rs`

Демонстрация работы с различными типами переменных, включая интерактивные, переменные окружения и переменные из файлов:
```bash
cargo run --example variable_demo
```

### `project_variables_test.rs`

Пример работы с переменными проекта через системные механизмы:
```bash
cargo run --example project_variables_test
```

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

### Запуск локальной конфигурации

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

### Запуск всех деплоев из конфигурации одновременно

```bash
cargo run -- -c ./examples/deploy-config.yml run -d all
```

### Запуск всех деплоев с конкретным событием

```bash
cargo run -- -c ./examples/deploy-config.yml run -d all -e deploy
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

## Примеры использования переменных

### Виды переменных

Deploy Commander поддерживает несколько типов переменных:

1. **Интерактивные переменные** - `{name}` - запрашиваются у пользователя во время выполнения
2. **Переменные окружения** - `{$VAR_NAME}` - берутся из окружения
3. **Переменные из файла** - `{#VAR_NAME}` - берутся из локального файла, указанного в `variables_file`
   - поддерживаются форматы JSON (`.json`) и YAML (`.yml`, `.yaml`)
   - формат определяется автоматически по расширению файла
4. **Глобальные переменные** - `{#GLOBAL_VAR}` - берутся из глобального файла, указанного в `settings.json`
   - глобальный файл также может быть в формате YAML

### Запуск примеров с различными типами переменных

```bash
# Интерактивный ввод переменных
cargo run -- -c ./examples/interactive-config-example.yml run -d interactive-demo -e interactive-mode

# Использование переменных окружения
cargo run -- -c ./examples/interactive-config-example.yml run -d interactive-demo -e env-variables-mode

# Использование переменных из JSON-файла
cargo run -- -c ./examples/interactive-config-example.yml run -d interactive-demo -e file-variables-mode

# Использование переменных из YAML-файла
cargo run -- -c ./examples/interactive-config-example.yml run -d interactive-demo -e yaml-variables-mode

# Использование глобальных переменных
cargo run -- -c ./examples/interactive-config-example.yml run -d interactive-demo -e global-variables-mode

# Смешанное использование переменных
cargo run -- -c ./examples/interactive-config-example.yml run -d interactive-demo -e mixed-variables-mode

# Использование предустановленных значений
cargo run -- -c ./examples/interactive-config-example.yml run -d interactive-demo -e preset-variables-mode

# Демонстрация приоритета переменных (локальные имеют приоритет над глобальными)
cargo run -- -c ./examples/interactive-config-example.yml run -d interactive-demo -e priority-variables-mode
```

### Примеры файлов переменных

#### JSON формат (variables.json)
```json
{
  "DB_HOST": "localhost", 
  "DB_PORT": "5432",
  "API_KEY": "test-api-key-123"
}
```

#### YAML формат (variables.yml)
```yaml
DB_HOST: localhost
DB_PORT: 5432
API_KEY: test-api-key-123
complex_value:
  nested: true
  count: 42
```

## Примеры запуска деплоев

### Запуск конкретного деплоя

```bash
# Запуск всех событий для деплоя backend-api
cargo run -- -c ./examples/deploy-config.yml run -d backend-api
```

### Запуск конкретного события деплоя

```bash
# Запуск только события deploy для деплоя backend-api
cargo run -- -c ./examples/deploy-config.yml run -d backend-api -e deploy
```

### Запуск всех деплоев одновременно

```bash
# Запуск всех деплоев из конфигурации
cargo run -- -c ./examples/deploy-config.yml run -d all
```

### Запуск всех деплоев с конкретным событием

```bash
# Запуск события deploy для всех деплоев
cargo run -- -c ./examples/deploy-config.yml run -d all -e deploy
``` 