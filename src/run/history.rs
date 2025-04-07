use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use log::info;
use std::time::{Duration, UNIX_EPOCH};

use crate::storage::{DeploymentHistory, DeploymentRecord};

/// Показывает историю деплоев с форматированием
pub fn display_deployment_history(
    history_path: &str,
    deployment_name: &str,
    limit: usize,
) -> Result<()> {
    let history = load_history(history_path)?;
    let records = history.get_records(deployment_name, limit);

    if records.is_empty() {
        println!("История деплоя '{}' пуста", deployment_name);
        return Ok(());
    }

    print_history_header(deployment_name, limit);
    for (i, record) in records.iter().enumerate() {
        print_history_record(i, record);
    }

    Ok(())
}

/// Загружает историю деплоев из файла
fn load_history(history_path: &str) -> Result<DeploymentHistory> {
    DeploymentHistory::load(history_path).with_context(|| {
        format!(
            "Не удалось загрузить историю деплоев из файла {}",
            history_path
        )
    })
}

/// Выводит заголовок истории деплоев
fn print_history_header(deployment_name: &str, limit: usize) {
    println!(
        "История деплоя '{}' (последние {} записей):",
        deployment_name, limit
    );
}

/// Выводит запись истории деплоя
fn print_history_record(index: usize, record: &DeploymentRecord) {
    let timestamp = format_timestamp(record.timestamp);
    let status = if record.success { "✅" } else { "❌" };
    let details = record.details.as_deref().unwrap_or("");

    println!(
        "{}. [{} UTC] {} {} {}",
        index + 1,
        timestamp,
        status,
        record.event,
        details
    );
}

/// Форматирует временную метку
fn format_timestamp(timestamp: u64) -> String {
    DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_secs(timestamp))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

/// Очищает историю деплоев
pub fn clear_history(history_path: &str, deployment_name: Option<&str>) -> Result<()> {
    crate::storage::clear_deployment_history(history_path, deployment_name).with_context(|| {
        let target = match deployment_name {
            Some(name) => format!("деплоя '{}'", name),
            None => "всех деплоев".to_string(),
        };
        format!("Не удалось очистить историю {}", target)
    })?;

    let success_message = match deployment_name {
        Some(name) => format!("История деплоя '{}' успешно очищена", name),
        None => "Вся история деплоев успешно очищена".to_string(),
    };

    info!("{}", success_message);
    println!("{}", success_message);

    Ok(())
}
