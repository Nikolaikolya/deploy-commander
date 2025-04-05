use crate::config::{Config, Command as ConfigCommand};
use crate::events::{EventEmitter, EventType};
use anyhow::{Context, Result};
use log::{error, info, warn};
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

pub async fn run_commands(config: &Config, deployment_name: &str, event_name: &str) -> Result<()> {
	// Находим деплой и событие
	let deployment = config.find_deployment(deployment_name)
		.with_context(|| format!("Деплой '{}' не найден", deployment_name))?;

	let event = deployment.events.iter()
		.find(|e| e.name == event_name)
		.with_context(|| format!("Событие '{}' не найдено в деплое '{}'",
														 event_name, deployment_name))?;

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
	let env_vars = deployment.environment.as_ref()
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
		).await;

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
		info!("Деплой '{}', событие '{}' успешно выполнено",
              deployment_name, event_name);
	} else {
		emitter.emit(EventType::DeploymentFailed {
			deployment: deployment_name.to_string(),
			event: event_name.to_string(),
		});
		error!("Деплой '{}', событие '{}' завершилось с ошибками",
               deployment_name, event_name);

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

	// Разбиваем команду на части
	let parts: Vec<&str> = cmd.command.trim().split_whitespace().collect();
	if parts.is_empty() {
		return Err(anyhow::anyhow!("Пустая команда"));
	}

	// Создаем процесс
	let mut command = Command::new(parts[0]);

	// Добавляем аргументы
	if parts.len() > 1 {
		command.args(&parts[1..]);
	}

	// Устанавливаем рабочую директорию, если указана
	if let Some(dir) = working_dir {
		if Path::new(dir).exists() {
			command.current_dir(dir);
		} else {
			warn!("Рабочая директория не существует: {}", dir);
		}
	}

	// Устанавливаем переменные окружения
	for (key, value) in env_vars {
		command.env(key, value);
	}

	// Настраиваем вывод
	command.stdout(Stdio::piped());
	command.stderr(Stdio::piped());

	// Выполняем команду с таймаутом
	let timeout_duration = Duration::from_secs(cmd.timeout.unwrap_or(300));

	let result = match timeout(timeout_duration, command.spawn()?.wait_with_output()).await {
		Ok(output_result) => output_result,
		Err(_) => {
			return Err(anyhow::anyhow!(
                "Превышено время ожидания выполнения команды ({} сек)",
                timeout_duration.as_secs()
            ));
		}
	};

	match result {
		Ok(output) => {
			// Выводим stdout и stderr
			let stdout = String::from_utf8_lossy(&output.stdout);
			let stderr = String::from_utf8_lossy(&output.stderr);

			if !stdout.is_empty() {
				info!("Вывод команды stdout:\n{}", stdout);
			}

			if !stderr.is_empty() {
				warn!("Вывод команды stderr:\n{}", stderr);
			}

			// Проверяем код возврата
			if output.status.success() {
				info!("Команда выполнена успешно");

				// Отправляем событие об успешном выполнении команды
				emitter.emit(EventType::CommandSucceeded {
					deployment: deployment_name.to_string(),
					event: event_name.to_string(),
					command: cmd.command.clone(),
					output: stdout.to_string(),
				});

				Ok(())
			} else {
				let exit_code = output.status.code().unwrap_or(-1);
				let error_msg = format!(
					"Команда завершилась с ненулевым кодом возврата: {}", exit_code
				);

				Err(anyhow::anyhow!(error_msg))
			}
		}
		Err(e) => {
			Err(anyhow::anyhow!("Ошибка выполнения команды: {}", e))
		}
	}
}