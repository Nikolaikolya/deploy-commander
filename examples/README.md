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

## Новый пример для демонстрации логирования

Был добавлен новый пример `command-logging-example.yml` для демонстрации возможностей логирования:

```bash
# Запуск демонстрации логирования стандартного вывода
cargo run -- -c ./examples/command-logging-example.yml run -d logging-demo -e stdout-demo

# Запуск демонстрации логирования потока ошибок
cargo run -- -c ./examples/command-logging-example.yml run -d logging-demo -e stderr-demo

# Запуск демонстрации перенаправления вывода в файл
cargo run -- -c ./examples/command-logging-example.yml run -d logging-demo -e file-redirect-demo

# Запуск демонстрации настройки директории логов
cargo run -- -c ./examples/command-logging-example.yml run -d logging-demo -e logs-dir-demo

# Запуск всей демонстрации логирования
cargo run -- -c ./examples/command-logging-example.yml run -d logging-demo
```

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
4. Стандартный вывод и вывод ошибок записываются в лог-файлы

## Настройка логирования команд

Deploy Commander поддерживает настройку директории для логов выполнения команд через файл настроек `settings.json`:

```json
{
    "log_file": "deploy-commander.log",
    "history_file": "deploy-history.json",
    "variables_file": "variables.json",
    "logs_dir": "logs"
}
```

Все команды логируются в единый файл `logs/YYYYMMDD_commands.log`, где YYYYMMDD - текущая дата. Каждая запись включает:
- Временную метку
- Информацию о деплое, событии и команде
- Статус выполнения (успех/ошибка)
- Полный вывод команды
- Сообщение об ошибке (если произошла ошибка)

Пример записи в лог-файле:
```
[22:29:39] Деплой: 'unicode-variables', Событие: 'test-vars', Команда: 'unicode-variables_test-vars_cmd_1'
Статус: Успех
Вывод:
Переменная из файла: значение на русском
--------------------------------------------------------------------------------
```

При каждом запуске программы проверяется наличие файла за текущую дату - если он существует, новые записи добавляются в конец файла, если нет - создается новый файл.

## Обработка вывода команд

Deploy Commander корректно обрабатывает вывод команд:

1. При успешном выполнении команды выводится стандартный вывод
2. При ошибке выводится сообщение об ошибке и стандартный вывод
3. Для многострочного вывода используется форматирование с отступами
4. Лог-файлы содержат полный вывод команды, включая все потоки

### Решение проблем с выводом

Если команда не корректно выводит в stderr/stdout и нужно получить полный вывод, можно использовать перенаправление вывода:

```yaml
- command: "команда > output.txt 2>&1 && cat output.txt"
  description: "Выполнение команды с перенаправлением вывода"
```

## Работа с кодировками

При работе с русскими символами в командах на Windows могут возникать проблемы с кодировками. Вот рекомендуемые способы обеспечить корректное отображение кириллицы:

### Для CMD

```yaml
- command: "cmd /c chcp 65001 > nul && echo Текст на русском языке"
  description: "Команда с корректной кодировкой UTF-8"
```

### Для PowerShell

```yaml
- command: "powershell -Command \"[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; Write-Host 'Текст на русском языке'\""
  description: "PowerShell команда с UTF-8 кодировкой"
```

### Многострочный вывод с кириллицей

```yaml
- command: "powershell -Command \"[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; Write-Host 'Строка 1'; Write-Host 'Строка 2'\""
  description: "Многострочный вывод с правильной кодировкой"
```

### Работа с переменными с кириллицей

Для корректной работы с переменными, содержащими русские символы:

```yaml
deployments:
  - name: demo
    variables_file: "variables.json"
    commands:
      - command: "cmd /c chcp 65001 > nul && echo Значение: {#русская_переменная}"
        description: "Вывод переменной с русскими символами"
```

Где содержимое файла `variables.json`:
```json
{
  "русская_переменная": "Значение на русском языке"
}
```

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
4. **Глобальные переменные** - `{#GLOBAL_VAR}` - берутся из глобального файла, указанного в `settings.json`

### Запуск примеров с различными типами переменных

```bash
# Интерактивный ввод переменных
cargo run -- -c ./examples/interactive-config-example.yml run -d interactive-demo -e interactive-mode

# Использование переменных окружения
cargo run -- -c ./examples/interactive-config-example.yml run -d interactive-demo -e env-variables-mode

# Использование переменных из файла
cargo run -- -c ./examples/interactive-config-example.yml run -d interactive-demo -e file-variables-mode

# Использование глобальных переменных
cargo run -- -c ./examples/interactive-config-example.yml run -d interactive-demo -e global-variables-mode

# Смешанное использование переменных
cargo run -- -c ./examples/interactive-config-example.yml run -d interactive-demo -e mixed-variables-mode

# Использование предустановленных значений
cargo run -- -c ./examples/interactive-config-example.yml run -d interactive-demo -e preset-variables-mode

# Демонстрация приоритета переменных (локальные имеют приоритет над глобальными)
cargo run -- -c ./examples/interactive-config-example.yml run -d interactive-demo -e priority-variables-mode
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