use anyhow::{Context, Result};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::run::deployments;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub deployments: Vec<Deployment>,
    pub variables_file: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            deployments: Vec::new(),
            variables_file: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Deployment {
    pub name: String,
    pub description: Option<String>,
    pub working_dir: Option<String>,
    pub environment: Option<Vec<String>>,
    /// Опциональный путь к файлу с переменными
    pub variables_file: Option<String>,
    pub events: Vec<Event>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    pub name: String,
    pub description: Option<String>,
    pub commands: Vec<Command>,
    pub fail_fast: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Command {
    pub command: String,
    pub description: Option<String>,
    pub ignore_errors: Option<bool>,
    pub rollback_command: Option<String>,
    /// Флаг интерактивного режима
    pub interactive: Option<bool>,
    /// Предопределенные ответы на запросы в интерактивном режиме
    /// Ключи - это текст или паттерн запроса, значения - ответы
    pub inputs: Option<HashMap<String, String>>,
    /// Опциональный путь к файлу с переменными для этой команды
    pub variables_file: Option<String>,
}

impl Config {
    /// Загружает конфигурацию из файла
    pub fn load(path: &str) -> Result<Self> {
        let config_path = Path::new(path);

        // Расширенное логирование для отладки проблем с путями
        let absolute_path = if let Ok(abs_path) = std::fs::canonicalize(config_path) {
            abs_path.to_string_lossy().to_string()
        } else {
            path.to_string()
        };

        info!(
            "Попытка загрузки конфигурации из '{}' (абсолютный путь: '{}')",
            path, absolute_path
        );

        if !config_path.exists() {
            // Дополнительное логирование с информацией о текущей директории
            let current_dir = if let Ok(dir) = std::env::current_dir() {
                dir.to_string_lossy().to_string()
            } else {
                "Не удалось получить текущую директорию".to_string()
            };

            error!(
                "Файл конфигурации не найден: '{}'. Текущая директория: '{}'.",
                path, current_dir
            );

            // Выводим содержимое директории для отладки
            if let Ok(entries) = std::fs::read_dir(Path::new(".")) {
                info!("Содержимое текущей директории:");
                for entry in entries {
                    if let Ok(entry) = entry {
                        info!("  {}", entry.path().display());
                    }
                }
            }

            // Если файл не существует, создаем пустую конфигурацию
            let config = Config {
                deployments: vec![],
                variables_file: None,
            };
            config.save(path)?;
            return Ok(config);
        }

        // Информация о найденном файле
        if let Ok(metadata) = std::fs::metadata(config_path) {
            info!(
                "Файл конфигурации найден: '{}', размер: {} байт",
                path,
                metadata.len()
            );

            // Если файл пустой или слишком маленький
            if metadata.len() < 10 {
                error!("Файл конфигурации существует, но может быть пустым или повреждён");

                // Выводим первые несколько байт для диагностики
                if let Ok(content) = fs::read(config_path) {
                    let preview = String::from_utf8_lossy(&content);
                    info!("Содержимое файла: '{}'", preview);
                }
            }
        }

        let content = match fs::read_to_string(config_path) {
            Ok(c) => {
                info!(
                    "Файл успешно прочитан, размер содержимого: {} байт",
                    c.len()
                );
                if c.len() < 50 {
                    info!("Предпросмотр содержимого: '{}'", c);
                } else {
                    info!("Предпросмотр содержимого (первые 50 байт): '{}'", &c[..50]);
                }
                c
            }
            Err(e) => {
                error!("Ошибка чтения файла: {}", e);
                return Err(anyhow::anyhow!(
                    "Не удалось прочитать файл конфигурации: {} ({})",
                    path,
                    e
                ));
            }
        };

        let config: Self = match serde_yaml::from_str(&content) {
            Ok(c) => {
                info!("YAML успешно десериализован");
                c
            }
            Err(e) => {
                error!("Ошибка десериализации YAML: {}", e);
                error!("Содержимое, вызвавшее ошибку: '{}'", content);
                return Err(anyhow::anyhow!(
                    "Неверный формат файла конфигурации: {} ({})",
                    path,
                    e
                ));
            }
        };

        info!("Конфигурация содержит {} деплоев", config.deployments.len());

        Ok(config)
    }

    /// Сохраняет конфигурацию в файл
    pub fn save(&self, path: &str) -> Result<()> {
        let yaml =
            serde_yaml::to_string(self).context("Не удалось сериализовать конфигурацию в YAML")?;

        fs::write(path, yaml)
            .with_context(|| format!("Не удалось записать конфигурацию в файл: {}", path))?;

        Ok(())
    }

    /// Находит деплоймент по имени
    pub fn find_deployment(&self, name: &str) -> Option<&Deployment> {
        self.deployments.iter().find(|d| d.name == name)
    }
}

/// Создает шаблон деплоя с указанным именем
pub fn create_template_deployment(name: &str, config_path: &str) -> Result<()> {
    let mut config = Config::load(config_path)?;

    // Проверяем, существует ли уже деплой с таким именем
    if config.find_deployment(name).is_some() {
        error!("Деплой с именем '{}' уже существует", name);
        return Err(anyhow::anyhow!("Деплой с таким именем уже существует"));
    }

    // Создаем шаблон деплоя
    let template = deployments::create_new_deployment(name);

    // Добавляем шаблон в конфигурацию
    config.deployments.push(template);

    // Сохраняем обновленную конфигурацию
    config.save(config_path)?;

    Ok(())
}

/// Проверяет конфигурацию деплоя на корректность
pub fn verify_deployment(config: &Config, deployment_name: &str) -> Result<bool> {
    let deployment = match config.find_deployment(deployment_name) {
        Some(d) => d,
        None => {
            error!("Деплой с именем '{}' не найден", deployment_name);
            return Ok(false);
        }
    };

    // Проверяем события деплоя
    deployments::validate_deployment_events(deployment)
}
