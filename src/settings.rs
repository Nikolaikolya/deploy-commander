/*!
# Модуль Settings

Модуль `settings` отвечает за работу с глобальными настройками приложения:

- Загрузка и сохранение настроек из JSON-файла
- Предоставление параметров для других модулей
- Управление путями к файлам логов и истории
- Управление путем к файлу глобальных переменных
*/

use anyhow::{Context, Result};
use log::info;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Константы по умолчанию
pub const DEFAULT_SETTINGS_PATH: &str = "settings.json";
pub const DEFAULT_LOG_FILE: &str = "deploy-commander.log";
pub const DEFAULT_HISTORY_FILE: &str = "deploy-history.json";
pub const DEFAULT_VARIABLES_FILE: &str = "variables.json";
pub const DEFAULT_LOGS_DIR: &str = "logs";

/// Структура глобальных настроек приложения
#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    /// Путь к файлу журнала
    pub log_file: String,

    /// Путь к файлу истории деплоев
    pub history_file: String,

    /// Путь к файлу глобальных переменных
    pub variables_file: String,

    /// Путь к директории логов команд
    pub logs_dir: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            log_file: DEFAULT_LOG_FILE.to_string(),
            history_file: DEFAULT_HISTORY_FILE.to_string(),
            variables_file: DEFAULT_VARIABLES_FILE.to_string(),
            logs_dir: DEFAULT_LOGS_DIR.to_string(),
        }
    }
}

impl Settings {
    /// Создает настройки с параметрами по умолчанию
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Загружает настройки из файла
    ///
    /// # Параметры
    ///
    /// * `path` - Путь к файлу настроек
    ///
    /// # Возвращаемое значение
    ///
    /// Настройки или ошибка загрузки
    pub fn load(path: &str) -> Result<Self> {
        if !Path::new(path).exists() {
            info!("Файл настроек не найден, создаем новый: {}", path);
            let settings = Self::default();
            settings.save(path)?;
            return Ok(settings);
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Не удалось прочитать файл настроек: {}", path))?;

        let settings = serde_json::from_str(&content)
            .with_context(|| format!("Неверный формат файла настроек: {}", path))?;

        // Проверяем, есть ли в настройках поле variables_file
        // Если нет, добавляем его и сохраняем обновленные настройки
        match upgrade_settings_if_needed(path, settings) {
            Ok(upgraded_settings) => Ok(upgraded_settings),
            Err(e) => {
                // Логируем ошибку и возвращаем исходные настройки
                info!("Не удалось обновить настройки: {}", e);
                Ok(Settings::load(path)?)
            }
        }
    }

    /// Сохраняет настройки в файл
    ///
    /// # Параметры
    ///
    /// * `path` - Путь к файлу настроек
    ///
    /// # Возвращаемое значение
    ///
    /// Результат сохранения или ошибка
    pub fn save(&self, path: &str) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .with_context(|| "Не удалось сериализовать настройки в JSON".to_string())?;

        fs::write(path, content)
            .with_context(|| format!("Не удалось сохранить файл настроек: {}", path))?;

        Ok(())
    }
}

/// Обновляет настройки, если они старой версии (без поля variables_file или logs_dir)
///
/// # Параметры
///
/// * `path` - Путь к файлу настроек
/// * `settings` - Текущие настройки
///
/// # Возвращаемое значение
///
/// Обновленные настройки или ошибка
fn upgrade_settings_if_needed(path: &str, mut settings: Settings) -> Result<Settings> {
    let content = fs::read_to_string(path)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;
    let json_obj = json.as_object().unwrap();
    let mut updated = false;

    // Если поле "variables_file" отсутствует, добавляем его
    if !json_obj.contains_key("variables_file") {
        info!("Обновление настроек: добавление поля variables_file");
        settings.variables_file = DEFAULT_VARIABLES_FILE.to_string();
        updated = true;
    }

    // Если поле "logs_dir" отсутствует, добавляем его
    if !json_obj.contains_key("logs_dir") {
        info!("Обновление настроек: добавление поля logs_dir");
        settings.logs_dir = DEFAULT_LOGS_DIR.to_string();
        updated = true;
    }

    // Сохраняем настройки, если они были обновлены
    if updated {
        settings.save(path)?;
        info!("Настройки успешно обновлены");
    }

    Ok(settings)
}

/// Получает настройки из файла или создаёт настройки по умолчанию
///
/// # Параметры
///
/// * `settings_path` - Путь к файлу настроек
///
/// # Возвращаемое значение
///
/// Настройки или ошибка
pub fn get_settings(settings_path: &str) -> Result<Settings> {
    Settings::load(settings_path)
}
