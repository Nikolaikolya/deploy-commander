use anyhow::Result;
use log::{error, info};

use command_system::command::ShellCommand;
use command_system::{
    ChainBuilder, ChainExecutionMode, CommandBuilder, CommandExecution, ConsoleLogger,
    ExecutionMode, LogLevel,
};

/// Создает команду Command System из строки
pub fn create_command(name: &str, command_str: &str) -> ShellCommand {
    CommandBuilder::new(name, command_str)
        .execution_mode(ExecutionMode::Sequential)
        .build()
}

/// Создает цепочку команд для последовательного выполнения
pub fn create_command_chain(
    name: &str,
    commands: Vec<(String, String)>,
) -> Result<command_system::CommandChain> {
    // Создаем логгер
    let logger = Box::new(ConsoleLogger::new(LogLevel::Info));

    // Создаем цепочку команд
    let mut chain = ChainBuilder::new(name)
        .execution_mode(ChainExecutionMode::Sequential)
        .logger(logger)
        .build();

    // Добавляем команды в цепочку
    for (cmd_name, cmd_str) in commands {
        let command = create_command(&cmd_name, &cmd_str);
        chain.add_command(command);
    }

    Ok(chain)
}

/// Выполняет команду в системе
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
pub fn validate_command(command: &str) -> Result<()> {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();
    if parts.is_empty() {
        return Err(anyhow::anyhow!("Пустая команда"));
    }

    // Проверяем, существует ли исполняемый файл
    let which_cmd = format!("which {}", parts[0]);

    match tokio::runtime::Runtime::new()?.block_on(execute_shell_command(&which_cmd)) {
        Ok(_) => Ok(()),
        Err(_) => Err(anyhow::anyhow!("Команда '{}' не найдена", parts[0])),
    }
}

/// Проверяет доступность необходимых команд
pub fn check_required_commands() -> Result<()> {
    let required = ["git", "docker", "ssh", "rsync"];

    for cmd in required {
        match validate_command(cmd) {
            Ok(_) => info!("Команда '{}' доступна", cmd),
            Err(e) => {
                error!("Требуемая команда '{}' недоступна: {}", cmd, e);
                return Err(anyhow::anyhow!("Отсутствует требуемая команда: {}", cmd));
            }
        }
    }

    Ok(())
}
