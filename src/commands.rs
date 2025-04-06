/*!
# Модуль Commands

Модуль `commands` содержит функции для работы с командами на уровне системы:

- Создание и запуск системных команд
- Выполнение shell-команд с заданными параметрами
- Обработка результатов выполнения команд
- Валидация команд перед выполнением

## Основные функции

- `create_command` - создает команду для выполнения с использованием Command System
- `execute_shell_command` - выполняет shell-команду и возвращает результат
- `validate_command` - проверяет доступность команды без ее выполнения
- `check_required_commands` - проверяет наличие всех необходимых инструментов
*/

use anyhow::Result;
use log::{error, info};

use command_system::command::ShellCommand;
use command_system::{CommandBuilder, CommandExecution, ExecutionMode};

/// Создает команду Command System из строки
///
/// # Параметры
///
/// * `name` - Имя команды для идентификации
/// * `command_str` - Строка с командой для выполнения
///
/// # Возвращаемое значение
///
/// Возвращает объект ShellCommand, настроенный для выполнения
pub fn create_command(name: &str, command_str: &str) -> ShellCommand {
    CommandBuilder::new(name, command_str)
        .execution_mode(ExecutionMode::Sequential)
        .build()
}

/// Выполняет команду в системе
///
/// # Параметры
///
/// * `command` - Строка с командой для выполнения
///
/// # Возвращаемое значение
///
/// Возвращает результат выполнения команды в виде строки или ошибку
pub async fn execute_shell_command(command: &str) -> Result<String> {
    info!("Выполнение команды: {}", command);

    let cmd_name = format!("cmd_{}", chrono::Utc::now().timestamp_millis());
    let command = create_command(&cmd_name, command);

    match command.execute().await {
        Ok(result) => {
            if result.success {
                Ok(result.output)
            } else {
                Err(anyhow::anyhow!(
                    "Команда завершилась с ошибкой: {}",
                    result
                        .error
                        .unwrap_or_else(|| "<неизвестная ошибка>".to_string())
                ))
            }
        }
        Err(e) => Err(anyhow::anyhow!("Ошибка выполнения команды: {}", e)),
    }
}

/// Валидирует команду без выполнения
///
/// # Параметры
///
/// * `command` - Строка с командой для проверки
///
/// # Возвращаемое значение
///
/// Возвращает Ok(()) если команда доступна, или ошибку если нет
pub async fn validate_command(command: &str) -> Result<()> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Err(anyhow::anyhow!("Пустая команда"));
    }

    // Проверяем, существует ли исполняемый файл
    #[cfg(target_family = "unix")]
    let which_cmd = format!("which {}", parts[0]);

    #[cfg(target_family = "windows")]
    let which_cmd = format!("where {}", parts[0]);

    match execute_shell_command(&which_cmd).await {
        Ok(_) => Ok(()),
        Err(_) => Err(anyhow::anyhow!("Команда '{}' не найдена", parts[0])),
    }
}

/// Проверяет доступность необходимых команд
///
/// Проверяет наличие git, docker, ssh и rsync в системе,
/// выводя предупреждение, если какие-то из них отсутствуют
///
/// # Возвращаемое значение
///
/// Всегда возвращает Ok(()), но логирует отсутствующие команды
pub async fn check_required_commands() -> Result<()> {
    let required = ["git", "docker", "ssh", "rsync"];
    let mut missing_commands = Vec::new();

    for cmd in required {
        match validate_command(cmd).await {
            Ok(_) => info!("Команда '{}' доступна", cmd),
            Err(e) => {
                error!("Команда '{}' недоступна: {}", cmd, e);
                missing_commands.push(cmd);
            }
        }
    }

    if !missing_commands.is_empty() {
        error!(
            "Отсутствуют некоторые команды: {}. Некоторые операции могут быть недоступны.",
            missing_commands.join(", ")
        );
    }

    Ok(())
}
