# Deploy Commander

Инструмент командной строки для автоматизации развертывания приложений с поддержкой
настраиваемых цепочек команд и автоматического отката при ошибках.

## Новые возможности

- **Поддержка YAML** - добавлена поддержка YAML-файлов для хранения переменных (*.yml, *.yaml)
- **Глобальные переменные** - поддержка глобального файла переменных для всех деплоев
- **Приоритетная подстановка переменных** - локальные переменные имеют приоритет над глобальными
- **Одновременный запуск всех деплоев** - возможность запустить все деплои командой `run -d all`
- **Улучшенная обработка ошибок** - более подробная информация об ошибках и причинах их возникновения

## Особенности

- Конфигурация деплоя через YAML-файлы
- Поддержка последовательного выполнения событий
- Автоматический откат при ошибках
- Ведение истории деплоев
- Проверка зависимостей перед запуском
- Настройка путей к файлам логов и истории
- Шаблоны для быстрого создания новых деплоев
- Поддержка шаблонных переменных в командах
- Возможность использования переменных из разных источников (интерактивный ввод, окружение, файлы)

## Возможности

- Управление командами деплоя через YAML конфигурацию
- Группировка команд по событиям (pre-deploy, deploy, post-deploy и др.)
- Последовательное выполнение команд с выводом результатов в консоль
- Использование библиотеки Command System для эффективного выполнения команд
- Формирование цепочек команд для каждого события
- Гибкая настройка рабочих директорий и переменных окружения
- Отслеживание и хранение истории деплоев
- Возможность запуска как отдельных событий, так и полного цикла деплоя
- Управление историей деплоев (просмотр, очистка)
- Использование шаблонных переменных в командах (интерактивный ввод, переменные окружения, переменные из файлов)

## Архитектура

Deploy Commander использует несколько паттернов проектирования, реализованных через библиотеку Command System:

- **Паттерн Команда** - для инкапсуляции запросов к системе
- **Паттерн Цепочка обязанностей** - для последовательного выполнения команд событий
- **Паттерн Строитель** - для конструирования команд и цепочек
- **Паттерн Визитор** - для логирования и отслеживания выполнения команд

## Установка

```bash
git clone https://github.com/yourusername/deploy-commander.git
cd deploy-commander
cargo build --release
```

## Использование

```bash
# Запуск деплоя
./target/release/deploy-cmd -c config.yml run -d myproject

# Запуск конкретного события деплоя
./target/release/deploy-cmd -c config.yml run -d myproject -e deploy

# Запуск всех деплоев из конфигурации одновременно
./target/release/deploy-cmd -c config.yml run -d all

# Запуск всех деплоев с конкретным событием
./target/release/deploy-cmd -c config.yml run -d all -e deploy

# Запуск с интерактивными переменными
./target/release/deploy-cmd -c examples/interactive-config-example.yml run -d interactive-demo -e interactive-mode

# Запуск с переменными из файла
./target/release/deploy-cmd -c examples/interactive-config-example.yml run -d interactive-demo -e file-variables-mode

# Запуск со смешанными переменными
./target/release/deploy-cmd -c examples/interactive-config-example.yml run -d interactive-demo -e mixed-variables-mode

# Просмотр доступных деплоев
./target/release/deploy-cmd -c config.yml list

# Создание нового шаблона деплоя
./target/release/deploy-cmd -c config.yml create -d newproject

# Просмотр истории деплоев
./target/release/deploy-cmd -c config.yml history -d myproject -l 10

# Очистка истории деплоев
./target/release/deploy-cmd -c config.yml clear-history -d myproject
```

## Конфигурация

### Файл настроек `settings.json`

Инструмент использует файл `settings.json` для хранения глобальных настроек:

```json
{
  "log_file": "deploy-commander.log",
  "history_file": "deploy-history.json",
  "variables_file": "variables.json"
}
```

### Файл конфигурации деплоя

Деплои настраиваются через YAML-конфигурацию:

```yaml
deployments:
  - name: myproject
    description: "Deployment of My Project"
    working_dir: "/var/www/myproject"
    environment:
      - "NODE_ENV=production"
      - "PORT=3000"
    variables_file: "./global_variables.json"
    events:
      - name: pre-deploy
        description: "Preparatory actions"
        commands:
          - command: "echo 'Starting deployment'"
            description: "Start message"
            ignore_errors: true

      - name: deploy
        description: "Main deployment process"
        commands:
          - command: "git pull origin main"
            description: "Get latest code"
            rollback_command: "git reset --hard HEAD~1"
            
          - command: "npm ci"
            description: "Install dependencies"
            
          - command: "npm run build"
            description: "Build the project"
            
      - name: variables-setup
        description: "Setup with variables"
        commands:
          - command: "echo 'Setting up {project_name} version {version}'"
            description: "Setup with interactive variables"
            interactive: true
            
          - command: "echo 'Environment: {$NODE_ENV}, Listening on port: {$PORT}'"
            description: "Using environment variables"
            interactive: true
            
          - command: "echo 'Database: {#DB_HOST}:{#DB_PORT}'"
            description: "Using variables from file"
            interactive: true
            variables_file: "./db_config.json"
            
      - name: post-deploy
        description: "Post-deployment actions"
        commands:
          - command: "pm2 restart app"
            description: "Restart application"
            rollback_command: "pm2 stop app"
            
          - command: "echo 'Deployment completed'"
            description: "End message"
            ignore_errors: true
        fail_fast: false
```

## Работа с переменными

Deploy Commander поддерживает несколько типов переменных для подстановки в команды:

1. **Интерактивные переменные** - значения запрашиваются у пользователя во время выполнения:
   ```yaml
   - command: "echo 'Привет, {name}!'"
     description: "Приветствие пользователя"
     interactive: true
   ```

2. **Переменные окружения** - значения берутся из окружения:
   ```yaml
   - command: "echo 'Среда: {$NODE_ENV}'"
     description: "Вывод переменной окружения"
     interactive: true
   ```

3. **Переменные из файла** - значения берутся из JSON или YAML файла:
   ```yaml
   - command: "echo 'Сервер: {#SERVER_URL}'"
     description: "Использование переменной из файла"
     interactive: true
     variables_file: "./server_config.json"
   ```
   
   Поддерживаются форматы JSON и YAML. Формат определяется автоматически по расширению файла 
   (`.json`, `.yml`, `.yaml`).

4. **Глобальные переменные** - значения берутся из глобального файла переменных:
   ```yaml
   - command: "echo 'Версия: {#GLOBAL_VERSION}'"
     description: "Использование глобальной переменной"
     interactive: true
   ```
   
   Глобальный файл переменных указывается в настройках приложения (settings.json):
   ```json
   {
     "log_file": "deploy-commander.log",
     "history_file": "deploy-history.json",
     "variables_file": "variables.json"
   }
   ```
   
   Глобальный файл переменных также может быть в формате YAML (например, `variables.yml`).

5. **Приоритет переменных** - локальные переменные имеют приоритет над глобальными. Если одна и та же переменная определена в локальном и глобальном файле, будет использовано локальное значение.

6. **Предустановленные значения** - можно предустановить значения для интерактивных переменных:
   ```yaml
   - command: "echo 'Привет, {name}!'"
     description: "Приветствие с предустановленным именем"
     interactive: true
     inputs:
       "name": "Пользователь"
   ```

7. **Смешанное использование** - можно комбинировать разные типы переменных:
   ```yaml
   - command: "echo 'Подключение к {#DB_HOST} под пользователем {$USER} в проекте {project_name}'"
     description: "Пример со смешанными переменными"
     interactive: true
     variables_file: "./db_config.json"
   ```

## Архитектура проекта

Проект имеет модульную структуру:

- `app` - Основная логика приложения
- `cli` - Интерфейс командной строки
- `config` - Работа с конфигурацией
- `commands` - Работа с системными командами
- `events` - Система событий и уведомлений
- `executor` - Выполнение команд и обработка ошибок
- `logging` - Настройка журналирования
- `run` - Управление процессом деплоя
- `settings` - Глобальные настройки приложения
- `storage` - Хранение и управление историей деплоев

## Лицензия

MIT