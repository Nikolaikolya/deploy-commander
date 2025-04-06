# Примеры использования Deploy Commander

В данной директории содержатся примеры файлов конфигурации и команд для использования Deploy Commander.

## Пример файла конфигурации

Файл `deploy-config.yml` содержит примеры конфигурации для различных типов деплоев:

1. `get-all-projects` - получение всех репозиториев
2. `frontend-app` - деплой фронтенд приложения
3. `backend-api` - деплой бэкенд API

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