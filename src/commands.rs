use anyhow::{Context, Result};
use log::{error, info};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

/// Выполняет команду в системе с выводом результатов в консоль
pub fn execute_shell_command(command: &str) -> Result<String> {
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

    // Настраиваем вывод для перехвата
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    // Выполняем команду
    let mut process = cmd
        .spawn()
        .with_context(|| format!("Ошибка запуска команды: {}", command))?;

    // Перехватываем и выводим stdout в реальном времени
    let stdout = process
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Не удалось получить stdout"))?;

    let stdout_reader = BufReader::new(stdout);
    let mut stdout_output = String::new();

    for line in stdout_reader.lines() {
        if let Ok(line) = line {
            println!("{}", line);
            stdout_output.push_str(&line);
            stdout_output.push('\n');
        }
    }

    // Ждем завершения процесса
    let status = process
        .wait()
        .with_context(|| format!("Ошибка при ожидании завершения команды: {}", command))?;

    // Проверяем код возврата
    if status.success() {
        Ok(stdout_output)
    } else {
        let exit_code = status.code().unwrap_or(-1);

        // Получаем stderr после завершения процесса
        let stderr = process
            .stderr
            .take()
            .ok_or_else(|| anyhow::anyhow!("Не удалось получить stderr"))?;

        let stderr_reader = BufReader::new(stderr);
        let mut stderr_output = String::new();

        for line in stderr_reader.lines() {
            if let Ok(line) = line {
                eprintln!("{}", line);
                stderr_output.push_str(&line);
                stderr_output.push('\n');
            }
        }

        Err(anyhow::anyhow!(
            "Команда завершилась с ошибкой (код {}): {}",
            exit_code,
            stderr_output
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

    match execute_shell_command(&which_cmd) {
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
