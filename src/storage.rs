/*!
# Модуль Storage

Модуль `storage` отвечает за хранение и управление историей деплоев:

- Сохранение и загрузка истории деплоев в JSON-формате
- Ведение записей о выполненных деплоях и их статусе
- Форматирование и отображение истории деплоев

## Основные компоненты

- `DeploymentHistory` - основной класс для работы с историей деплоев
- `DeploymentRecord` - запись о выполнении деплоя или его части
- `record_deployment` - функция для записи события деплоя
- `record_chain_result` - функция для записи результата выполнения цепочки команд
*/

use anyhow::{Context, Result};
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use command_system::chain::command_chain::ChainResult as ChainExecutionResult;
use command_system::command::CommandResult;

/// Структура для хранения истории деплоев
#[derive(Debug, Serialize, Deserialize)]
pub struct DeploymentHistory {
    /// Записи истории, сгруппированные по имени деплоя
    records: HashMap<String, Vec<DeploymentRecord>>,
}

/// Запись в истории деплоев
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeploymentRecord {
    /// Имя деплоя
    pub deployment: String,
    /// Имя события
    pub event: String,
    /// Временная метка (UNIX timestamp)
    pub timestamp: u64,
    /// Успешно ли выполнение
    pub success: bool,
    /// Дополнительные детали (опционально)
    pub details: Option<String>,
}

impl DeploymentHistory {
    /// Создает новую пустую историю деплоев
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
        }
    }

    /// Загружает историю деплоев из файла
    ///
    /// # Параметры
    ///
    /// * `path` - Путь к файлу истории
    ///
    /// # Возвращаемое значение
    ///
    /// История деплоев или ошибка загрузки
    pub fn load(path: &str) -> Result<Self> {
        if !Path::new(path).exists() {
            info!("История деплоев не найдена, создаем новую: {}", path);
            return Ok(Self::new());
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Не удалось прочитать файл истории деплоев: {}", path))?;

        serde_json::from_str(&content)
            .with_context(|| format!("Неверный формат файла истории деплоев: {}", path))
    }

    /// Сохраняет историю деплоев в файл
    ///
    /// # Параметры
    ///
    /// * `path` - Путь к файлу истории
    ///
    /// # Возвращаемое значение
    ///
    /// Результат сохранения или ошибка
    pub fn save(&self, path: &str) -> Result<()> {
        let file = File::create(path)
            .with_context(|| format!("Не удалось создать файл истории деплоев: {}", path))?;

        serde_json::to_writer_pretty(file, &self)
            .with_context(|| "Не удалось сериализовать историю деплоев в JSON".to_string())?;

        Ok(())
    }

    /// Добавляет запись в историю деплоев
    ///
    /// # Параметры
    ///
    /// * `record` - Запись для добавления
    pub fn add_record(&mut self, record: DeploymentRecord) {
        self.records
            .entry(record.deployment.clone())
            .or_default()
            .push(record);
    }

    /// Получает записи истории для заданного деплоя
    ///
    /// # Параметры
    ///
    /// * `deployment` - Имя деплоя
    /// * `limit` - Максимальное количество записей
    ///
    /// # Возвращаемое значение
    ///
    /// Вектор записей истории, ограниченный указанным лимитом
    pub fn get_records(&self, deployment: &str, limit: usize) -> Vec<&DeploymentRecord> {
        self.records
            .get(deployment)
            .map(|records| {
                let start = if records.len() > limit {
                    records.len() - limit
                } else {
                    0
                };
                records[start..].iter().collect()
            })
            .unwrap_or_default()
    }

    /// Очищает историю для указанного деплоя
    ///
    /// # Параметры
    ///
    /// * `deployment` - Имя деплоя для очистки
    pub fn clear_deployment(&mut self, deployment: &str) {
        self.records.remove(deployment);
    }

    /// Очищает всю историю деплоев
    pub fn clear_all(&mut self) {
        self.records.clear();
    }
}

/// Записывает событие деплоя в историю
///
/// # Параметры
///
/// * `path` - Путь к файлу истории
/// * `deployment` - Имя деплоя
/// * `event` - Имя события
/// * `success` - Успешность выполнения
/// * `details` - Дополнительные детали (опционально)
///
/// # Возвращаемое значение
///
/// Результат записи или ошибка
pub fn record_deployment(
    path: &str,
    deployment: &str,
    event: &str,
    success: bool,
    details: Option<String>,
) -> Result<()> {
    let mut history = DeploymentHistory::load(path)?;

    let record = DeploymentRecord {
        deployment: deployment.to_string(),
        event: event.to_string(),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        success,
        details,
    };

    history.add_record(record);
    history.save(path)?;

    Ok(())
}

/// Очищает историю деплоев
///
/// # Параметры
///
/// * `path` - Путь к файлу истории
/// * `deployment` - Имя деплоя (если None, очищается вся история)
///
/// # Возвращаемое значение
///
/// Результат очистки или ошибка
pub fn clear_deployment_history(path: &str, deployment: Option<&str>) -> Result<()> {
    let mut history = DeploymentHistory::load(path)?;

    match deployment {
        Some(dep) => {
            history.clear_deployment(dep);
        }
        None => {
            history.clear_all();
        }
    }

    history.save(path)?;

    Ok(())
}

/// Записывает результат выполнения цепочки команд в историю
///
/// # Параметры
///
/// * `path` - Путь к файлу истории
/// * `deployment` - Имя деплоя
/// * `event` - Имя события
/// * `result` - Результат выполнения цепочки команд
///
/// # Возвращаемое значение
///
/// Результат записи или ошибка
pub fn record_chain_result(
    path: &str,
    deployment: &str,
    event: &str,
    result: &ChainExecutionResult,
) -> Result<()> {
    let success = result.success;
    let details = if success {
        Some(format!("Успешно выполнено {} команд", result.results.len()))
    } else {
        match &result.error {
            Some(error) => Some(error.clone()),
            None => {
                let failed_command = result
                    .results
                    .iter()
                    .find(|r| !r.success)
                    .map(format_command_result)
                    .unwrap_or_else(|| "Неизвестная ошибка".to_string());

                Some(failed_command)
            }
        }
    };

    record_deployment(path, deployment, event, success, details)
}

/// Форматирует результат выполнения команды в строку
///
/// # Параметры
///
/// * `result` - Результат выполнения команды
///
/// # Возвращаемое значение
///
/// Отформатированная строка с результатом
fn format_command_result(result: &CommandResult) -> String {
    if result.success {
        format!("Команда успешно выполнена: {}", result.output)
    } else {
        format!(
            "Ошибка команды: {}",
            result
                .error
                .clone()
                .unwrap_or_else(|| "<неизвестная ошибка>".to_string())
        )
    }
}
