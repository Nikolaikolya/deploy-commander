use crate::commands::execute_shell_command;
use crate::config::{Command as ConfigCommand, Config};
use crate::events::{EventEmitter, EventType};
use anyhow::{Context, Result};
use log::{error, info, warn};
use std::path::Path;

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

    // Выполняем команды
    let fail_fast = event.fail_fast.unwrap_or(true);
    let mut success = true;

    for (i, cmd) in event.commands.iter().enumerate() {
        let result = execute_command(
            cmd,
            working_dir,
            &env_vars,
            i + 1,
            event.commands.len(),
            &emitter,
            deployment_name,
            event_name,
        )
        .await;

        if let Err(e) = result {
            error!("Ошибка выполнения команды: {}", e);

            // Отправляем событие об ошибке
            emitter.emit(EventType::CommandFailed {
                deployment: deployment_name.to_string(),
                event: event_name.to_string(),
                command: cmd.command.clone(),
                error: e.to_string(),
            });

            // Проверяем, нужно ли игнорировать ошибку
            if !cmd.ignore_errors.unwrap_or(false) {
                success = false;

                // Если fail_fast, прерываем выполнение
                if fail_fast {
                    error!("Остановка деплоя из-за ошибки и настройки fail_fast=true");
                    break;
                }
            } else {
                warn!("Игнорирование ошибки в команде (ignore_errors=true)");
            }
        }
    }

    // Отправляем событие о завершении выполнения
    if success {
        emitter.emit(EventType::DeploymentSucceeded {
            deployment: deployment_name.to_string(),
            event: event_name.to_string(),
        });
        info!(
            "Деплой '{}', событие '{}' успешно выполнено",
            deployment_name, event_name
        );
    } else {
        emitter.emit(EventType::DeploymentFailed {
            deployment: deployment_name.to_string(),
            event: event_name.to_string(),
        });
        error!(
            "Деплой '{}', событие '{}' завершилось с ошибками",
            deployment_name, event_name
        );

        return Err(anyhow::anyhow!("Деплой завершился с ошибками"));
    }

    Ok(())
}

async fn execute_command(
    cmd: &ConfigCommand,
    working_dir: Option<&str>,
    env_vars: &[(String, String)],
    index: usize,
    total: usize,
    emitter: &EventEmitter,
    deployment_name: &str,
    event_name: &str,
) -> Result<()> {
    info!("[{}/{}] Выполнение команды: {}", index, total, cmd.command);

    if let Some(desc) = &cmd.description {
        info!("Описание: {}", desc);
    }

    // Отправляем событие о начале выполнения команды
    emitter.emit(EventType::CommandStarted {
        deployment: deployment_name.to_string(),
        event: event_name.to_string(),
        command: cmd.command.clone(),
        index,
        total,
    });

    // Меняем рабочую директорию, если указана
    let current_dir = std::env::current_dir()?;
    if let Some(dir) = working_dir {
        if Path::new(dir).exists() {
            std::env::set_current_dir(dir)?;
            info!("Изменена рабочая директория на: {}", dir);
        } else {
            warn!("Рабочая директория не существует: {}", dir);
        }
    }

    // Настраиваем переменные окружения
    let original_env_vars = env_vars
        .iter()
        .map(|(key, _)| (key.clone(), std::env::var(key).ok()))
        .collect::<Vec<_>>();

    // Устанавливаем временные переменные окружения
    for (key, value) in env_vars {
        std::env::set_var(key, value);
    }

    // Выполняем команду
    let result = execute_shell_command(&cmd.command);

    // Восстанавливаем переменные окружения
    for (key, original_value) in original_env_vars {
        match original_value {
            Some(value) => std::env::set_var(&key, value),
            None => std::env::remove_var(&key),
        }
    }

    // Восстанавливаем рабочую директорию
    if working_dir.is_some() {
        std::env::set_current_dir(current_dir)?;
    }

    match result {
        Ok(output) => {
            info!("Команда выполнена успешно");

            // Отправляем событие об успешном выполнении команды
            emitter.emit(EventType::CommandSucceeded {
                deployment: deployment_name.to_string(),
                event: event_name.to_string(),
                command: cmd.command.clone(),
                output,
            });

            Ok(())
        }
        Err(e) => {
            error!("Ошибка выполнения команды: {}", e);

            Err(anyhow::anyhow!("Ошибка выполнения команды: {}", e))
        }
    }
}
