use crate::config::Config;
use crate::events::{EventEmitter, EventType};
use crate::storage;
use anyhow::{Context, Result};
use command_system::{ChainBuilder, ChainExecutionMode, CommandBuilder, ConsoleLogger, LogLevel};
use log::{error, info, warn};

pub async fn run_commands(config: &Config, deployment_name: &str, event_name: &str) -> Result<()> {
    // Находим деплой и событие
    let deployment = config
        .find_deployment(deployment_name)
        .with_context(|| format!("Деплой '{}' не найден", deployment_name))?;

    let event = deployment
        .events
        .iter()
        .find(|e| e.name == event_name)
        .with_context(|| {
            format!(
                "Событие '{}' не найдено в деплое '{}'",
                event_name, deployment_name
            )
        })?;

    // Создаем эмиттер событий
    let emitter = EventEmitter::new();

    // Отправляем событие о начале выполнения
    emitter.emit(EventType::DeploymentStarted {
        deployment: deployment_name.to_string(),
        event: event_name.to_string(),
    });

    // Определяем рабочую директорию
    let working_dir = deployment.working_dir.as_deref();

    // Определяем переменные окружения
    let env_vars = deployment
        .environment
        .as_ref()
        .map(|vars| {
            vars.iter()
                .filter_map(|var| {
                    let parts: Vec<&str> = var.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        Some((parts[0].to_string(), parts[1].to_string()))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    // Создаем и настраиваем цепочку команд
    // Создаем логгер
    let logger = Box::new(ConsoleLogger::new(LogLevel::Info));

    // Создаем цепочку команд
    let mut chain = ChainBuilder::new(&format!("{}_{}_chain", deployment_name, event_name))
        .execution_mode(ChainExecutionMode::Sequential)
        .logger(logger)
        .build();

    // Собираем команды в цепочку с учетом рабочей директории и переменных окружения
    for (idx, cmd) in event.commands.iter().enumerate() {
        let mut command_builder = CommandBuilder::new(
            &format!("{}_{}_cmd_{}", deployment_name, event_name, idx + 1),
            &cmd.command,
        );

        // Устанавливаем рабочую директорию
        if let Some(dir) = working_dir {
            command_builder = command_builder.working_dir(dir);
        }

        // Устанавливаем переменные окружения
        for (key, value) in &env_vars {
            command_builder = command_builder.env_var(key, value);
        }

        // Добавляем команду в цепочку
        chain.add_command(command_builder.build());
    }

    // Выполняем цепочку команд
    let result = chain.execute().await;

    // Путь к файлу истории деплоев
    let history_path = "deploy-history.json";

    // Проверяем результат выполнения
    match result {
        Ok(chain_result) => {
            // Записываем результат в историю
            if let Err(e) = storage::record_chain_result(
                history_path,
                deployment_name,
                event_name,
                &chain_result,
            ) {
                warn!("Ошибка записи результата в историю: {}", e);
            }

            if chain_result.success {
                // Все команды выполнены успешно
                emitter.emit(EventType::DeploymentSucceeded {
                    deployment: deployment_name.to_string(),
                    event: event_name.to_string(),
                });
                info!(
                    "Деплой '{}', событие '{}' успешно выполнено",
                    deployment_name, event_name
                );

                Ok(())
            } else {
                // Произошла ошибка в одной из команд
                let error_msg = chain_result
                    .error
                    .unwrap_or_else(|| "Неизвестная ошибка".to_string());

                emitter.emit(EventType::DeploymentFailed {
                    deployment: deployment_name.to_string(),
                    event: event_name.to_string(),
                });
                error!(
                    "Деплой '{}', событие '{}' завершилось с ошибками: {}",
                    deployment_name, event_name, error_msg
                );

                Err(anyhow::anyhow!(
                    "Деплой завершился с ошибками: {}",
                    error_msg
                ))
            }
        }
        Err(e) => {
            // Критическая ошибка выполнения цепочки
            emitter.emit(EventType::DeploymentFailed {
                deployment: deployment_name.to_string(),
                event: event_name.to_string(),
            });
            error!(
                "Критическая ошибка выполнения деплоя '{}', событие '{}': {}",
                deployment_name, event_name, e
            );

            // Записываем ошибку в историю
            if let Err(log_err) = storage::record_deployment(
                history_path,
                deployment_name,
                &format!("error-{}", event_name),
                false,
                Some(e.to_string()),
            ) {
                warn!("Ошибка записи события: {}", log_err);
            }

            Err(anyhow::anyhow!("Критическая ошибка деплоя: {}", e))
        }
    }
}
