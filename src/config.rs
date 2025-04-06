use anyhow::{Context, Result};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub deployments: Vec<Deployment>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Deployment {
    pub name: String,
    pub description: Option<String>,
    pub working_dir: Option<String>,
    pub environment: Option<Vec<String>>,
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
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let config_path = Path::new(path);

        if !config_path.exists() {
            // Если файл не существует, создаем пустую конфигурацию
            info!("Файл конфигурации не найден, создаем новый: {}", path);
            let config = Config {
                deployments: vec![],
            };
            config.save(path)?;
            return Ok(config);
        }

        let content = fs::read_to_string(config_path)
            .with_context(|| format!("Не удалось прочитать файл конфигурации: {}", path))?;

        serde_yaml::from_str(&content)
            .with_context(|| format!("Неверный формат файла конфигурации: {}", path))
    }

    pub fn save(&self, path: &str) -> Result<()> {
        let yaml =
            serde_yaml::to_string(self).context("Не удалось сериализовать конфигурацию в YAML")?;

        fs::write(path, yaml)
            .with_context(|| format!("Не удалось записать конфигурацию в файл: {}", path))?;

        Ok(())
    }

    pub fn find_deployment(&self, name: &str) -> Option<&Deployment> {
        self.deployments.iter().find(|d| d.name == name)
    }

    pub fn find_event<'a>(&'a self, deployment_name: &str, event_name: &str) -> Option<&'a Event> {
        self.find_deployment(deployment_name)
            .and_then(|d| d.events.iter().find(|e| e.name == event_name))
    }
}

pub fn create_template_deployment(name: &str, config_path: &str) -> Result<()> {
    let mut config = Config::load(config_path)?;

    // Проверяем, существует ли уже деплой с таким именем
    if config.find_deployment(name).is_some() {
        error!("Деплой с именем '{}' уже существует", name);
        return Err(anyhow::anyhow!("Деплой с таким именем уже существует"));
    }

    // Создаем шаблон деплоя
    let template = Deployment {
        name: name.to_string(),
        description: Some(format!("Деплой {}", name)),
        working_dir: Some("/var/www/app".to_string()),
        environment: Some(vec![
            "NODE_ENV=production".to_string(),
            "PORT=3000".to_string(),
        ]),
        events: vec![
            Event {
                name: "pre-deploy".to_string(),
                description: Some("Команды перед деплоем".to_string()),
                commands: vec![Command {
                    command: "echo 'Начало деплоя'".to_string(),
                    description: Some("Вывод сообщения о начале деплоя".to_string()),
                    ignore_errors: Some(true),
                    rollback_command: None,
                }],
                fail_fast: Some(true),
            },
            Event {
                name: "deploy".to_string(),
                description: Some("Основные команды деплоя".to_string()),
                commands: vec![
                    Command {
                        command: "git pull origin main".to_string(),
                        description: Some(
                            "Получение последних изменений из репозитория".to_string(),
                        ),
                        ignore_errors: None,
                        rollback_command: None,
                    },
                    Command {
                        command: "npm ci".to_string(),
                        description: Some("Установка зависимостей".to_string()),
                        ignore_errors: None,
                        rollback_command: None,
                    },
                    Command {
                        command: "npm run build".to_string(),
                        description: Some("Сборка проекта".to_string()),
                        ignore_errors: None,
                        rollback_command: None,
                    },
                ],
                fail_fast: Some(true),
            },
            Event {
                name: "post-deploy".to_string(),
                description: Some("Команды после деплоя".to_string()),
                commands: vec![
                    Command {
                        command: "pm2 restart app".to_string(),
                        description: Some("Перезапуск приложения".to_string()),
                        ignore_errors: None,
                        rollback_command: None,
                    },
                    Command {
                        command: "echo 'Деплой завершен'".to_string(),
                        description: Some("Вывод сообщения о завершении деплоя".to_string()),
                        ignore_errors: Some(true),
                        rollback_command: None,
                    },
                ],
                fail_fast: Some(false),
            },
        ],
    };

    // Добавляем шаблон в конфигурацию
    config.deployments.push(template);

    // Сохраняем обновленную конфигурацию
    config.save(config_path)?;

    Ok(())
}

pub fn verify_deployment(config: &Config, deployment_name: &str) -> Result<bool> {
    let deployment = match config.find_deployment(deployment_name) {
        Some(d) => d,
        None => {
            error!("Деплой с именем '{}' не найден", deployment_name);
            return Ok(false);
        }
    };

    // Проверка на пустые события
    if deployment.events.is_empty() {
        error!("Деплой '{}' не содержит событий", deployment_name);
        return Ok(false);
    }

    // Проверка каждого события на наличие команд
    for event in &deployment.events {
        if event.commands.is_empty() {
            error!(
                "Событие '{}' в деплое '{}' не содержит команд",
                event.name, deployment_name
            );
            return Ok(false);
        }
    }

    Ok(true)
}
