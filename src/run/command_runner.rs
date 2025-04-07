use anyhow::Result;
use log::{error, info, warn};
use std::path::Path;

use crate::config::Config;
use crate::executor;
use crate::settings;
use crate::storage;

/// Выполняет команды для указанного деплоя и события
///
/// # Параметры
///
/// * `config` - Конфигурация деплоя
/// * `deployment_name` - Имя деплоя
/// * `event_name` - Имя события
/// * `history_path` - Путь к файлу истории деплоев
///
/// # Возвращаемое значение
///
/// Результат выполнения команды или ошибка
pub async fn execute_command(
    config: &Config,
    deployment_name: &str,
    event_name: &str,
    history_path: &str,
) -> Result<()> {
    info!(
        "Выполнение команд для деплоя '{}', событие '{}'",
        deployment_name, event_name
    );

    // Получаем настройки и путь к глобальному файлу переменных
    let settings = settings::get_settings(settings::DEFAULT_SETTINGS_PATH).unwrap_or_default();
    let global_variables_file = if Path::new(&settings.variables_file).exists() {
        Some(settings.variables_file.as_str())
    } else {
        None
    };

    if let Some(file) = global_variables_file {
        info!("Используется глобальный файл переменных: {}", file);
    }

    // Записываем событие начала деплоя
    if let Err(e) = crate::storage::record_deployment(
        history_path,
        deployment_name,
        &format!("start-{}", event_name),
        true,
        None,
    ) {
        info!("Ошибка записи события: {}", e);
    }

    // Вызываем выполнение команд из executor
    match executor::run_commands(config, deployment_name, event_name, global_variables_file).await {
        Ok(_) => {
            info!(
                "Деплой '{}', событие '{}' успешно выполнено",
                deployment_name, event_name
            );

            // Записываем успешное завершение
            if let Err(e) = record_success(
                history_path,
                deployment_name,
                &format!("complete-{}", event_name),
                None,
            ) {
                info!("Ошибка записи события: {}", e);
            }

            Ok(())
        }
        Err(e) => {
            error!(
                "Ошибка выполнения команд для деплоя '{}', событие '{}': {}",
                deployment_name, event_name, e
            );

            // Записываем ошибку
            if let Err(log_err) = record_failure(
                history_path,
                deployment_name,
                &format!("failed-{}", event_name),
                e.to_string(),
            ) {
                info!("Ошибка записи события: {}", log_err);
            }

            Err(e)
        }
    }
}

/// Записывает информацию об успешном выполнении команды
pub fn record_success(
    history_path: &str,
    deployment: &str,
    event: &str,
    details: Option<String>,
) -> Result<()> {
    let details = details.unwrap_or_else(|| "Успешно выполнено".to_string());
    if let Err(e) = storage::record_deployment(
        history_path,
        deployment,
        &format!("success-{}", event),
        true,
        Some(details),
    ) {
        warn!("Ошибка записи успешного события: {}", e);
        return Err(anyhow::anyhow!("Ошибка записи успешного события: {}", e));
    }
    Ok(())
}

/// Записывает информацию о неудачном выполнении команды
pub fn record_failure(
    history_path: &str,
    deployment: &str,
    event: &str,
    error_msg: String,
) -> Result<()> {
    if let Err(e) = storage::record_deployment(
        history_path,
        deployment,
        &format!("failed-{}", event),
        false,
        Some(error_msg),
    ) {
        warn!("Ошибка записи неудачного события: {}", e);
        return Err(anyhow::anyhow!("Ошибка записи неудачного события: {}", e));
    }
    Ok(())
}
