use anyhow::{Context, Result};
use log::{error, info};
use std::process::{Command, Stdio};

/// Выполняет команду в системе с таймаутом
pub fn execute_shell_command(command: &str, timeout_secs: u64) -> Result<String> {
	info!("Выполнение команды: {}", command);

	// Разделяем команду на части
	let parts: Vec<&str> = command.trim().split_whitespace().collect();
	if parts.is_empty() {
		return Err(anyhow::anyhow!("Пустая команда"));
	}

	// Создаем процесс
	let mut cmd = Command::new(parts[0]);

	// Добавляем аргументы
	if parts.len() > 1 {
		cmd.args(&parts[1..]);
	}

	// Настраиваем вывод
	cmd.stdout(Stdio::piped());
	cmd.stderr(Stdio::piped());

	// Выполняем команду
	// Примечание: в реальном приложении следует использовать
	// асинхронные команды с таймаутом через tokio
	let output = cmd.output()
		.with_context(|| format!("Ошибка выполнения команды: {}", command))?;

	// Проверяем код возврата
	if output.status.success() {
		// Преобразуем вывод в строку
		let stdout = String::from_utf8_lossy(&output.stdout).to_string();
		Ok(stdout)
	} else {
		let exit_code = output.status.code().unwrap_or(-1);
		let stderr = String::from_utf8_lossy(&output.stderr).to_string();

		Err(anyhow::anyhow!(
            "Команда завершилась с ошибкой (код {}): {}",
            exit_code,
            stderr
        ))
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

	match execute_shell_command(&which_cmd, 5) {
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