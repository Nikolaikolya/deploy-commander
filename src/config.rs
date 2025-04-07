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

        if !config_path.exists() {
            // Если файл не существует, создаем пустую конфигурацию
            info!("Файл конфигурации не найден, создаем новый: {}", path);
            let config = Config {
                deployments: vec![],
                variables_file: None,
            };
            config.save(path)?;
            return Ok(config);
        }

        let content = fs::read_to_string(config_path)
            .with_context(|| format!("Не удалось прочитать файл конфигурации: {}", path))?;

        serde_yaml::from_str(&content)
            .with_context(|| format!("Неверный формат файла конфигурации: {}", path))
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
