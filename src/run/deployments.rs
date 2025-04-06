use anyhow::{Context, Result};
use log::error;

use crate::config::Config;
use crate::config::Deployment;

/// Модуль с шаблонами для создания деплоев
pub mod templates {
    use crate::config;

    // Константы для шаблонов деплоя
    const DEFAULT_WORKING_DIR: &str = "/var/www/app";
    const DEFAULT_ENV_VARS: [&str; 2] = ["NODE_ENV=production", "PORT=3000"];

    // Константы для команд пре-деплоя
    const PRE_DEPLOY_NAME: &str = "pre-deploy";
    const PRE_DEPLOY_DESC: &str = "Команды перед деплоем";
    const PRE_DEPLOY_CMD: &str = "echo 'Начало деплоя'";
    const PRE_DEPLOY_CMD_DESC: &str = "Вывод сообщения о начале деплоя";

    // Константы для команд деплоя
    const DEPLOY_NAME: &str = "deploy";
    const DEPLOY_DESC: &str = "Основные команды деплоя";
    const DEPLOY_GIT_CMD: &str = "git pull origin main";
    const DEPLOY_GIT_DESC: &str = "Получение последних изменений из репозитория";
    const DEPLOY_GIT_ROLLBACK: &str = "git reset --hard HEAD~1";
    const DEPLOY_DEPS_CMD: &str = "npm ci";
    const DEPLOY_DEPS_DESC: &str = "Установка зависимостей";
    const DEPLOY_BUILD_CMD: &str = "npm run build";
    const DEPLOY_BUILD_DESC: &str = "Сборка проекта";

    // Константы для команд пост-деплоя
    const POST_DEPLOY_NAME: &str = "post-deploy";
    const POST_DEPLOY_DESC: &str = "Команды после деплоя";
    const POST_DEPLOY_RESTART_CMD: &str = "pm2 restart app";
    const POST_DEPLOY_RESTART_DESC: &str = "Перезапуск приложения";
    const POST_DEPLOY_RESTART_ROLLBACK: &str = "pm2 stop app";
    const POST_DEPLOY_FINISH_CMD: &str = "echo 'Деплой завершен'";
    const POST_DEPLOY_FINISH_DESC: &str = "Вывод сообщения о завершении деплоя";

    /// Создает новый деплой с шаблонными настройками
    pub fn create_new_deployment(name: &str) -> config::Deployment {
        config::Deployment {
            name: name.to_string(),
            description: Some(format!("Деплой {}", name)),
            working_dir: Some(DEFAULT_WORKING_DIR.to_string()),
            environment: Some(DEFAULT_ENV_VARS.iter().map(|&s| s.to_string()).collect()),
            variables_file: None,
            events: vec![
                create_pre_deploy_event(),
                create_deploy_event(),
                create_post_deploy_event(),
            ],
        }
    }

    /// Создает стандартное событие "pre-deploy"
    pub fn create_pre_deploy_event() -> config::Event {
        config::Event {
            name: PRE_DEPLOY_NAME.to_string(),
            description: Some(PRE_DEPLOY_DESC.to_string()),
            commands: vec![config::Command {
                command: PRE_DEPLOY_CMD.to_string(),
                description: Some(PRE_DEPLOY_CMD_DESC.to_string()),
                ignore_errors: Some(true),
                rollback_command: None,
                interactive: Some(false),
                inputs: None,
                variables_file: None,
            }],
            fail_fast: Some(true),
        }
    }

    /// Создает стандартное событие "deploy"
    pub fn create_deploy_event() -> config::Event {
        config::Event {
            name: DEPLOY_NAME.to_string(),
            description: Some(DEPLOY_DESC.to_string()),
            commands: vec![
                config::Command {
                    command: DEPLOY_GIT_CMD.to_string(),
                    description: Some(DEPLOY_GIT_DESC.to_string()),
                    ignore_errors: None,
                    rollback_command: Some(DEPLOY_GIT_ROLLBACK.to_string()),
                    interactive: Some(false),
                    inputs: None,
                    variables_file: None,
                },
                config::Command {
                    command: DEPLOY_DEPS_CMD.to_string(),
                    description: Some(DEPLOY_DEPS_DESC.to_string()),
                    ignore_errors: None,
                    rollback_command: None,
                    interactive: Some(false),
                    inputs: None,
                    variables_file: None,
                },
                config::Command {
                    command: DEPLOY_BUILD_CMD.to_string(),
                    description: Some(DEPLOY_BUILD_DESC.to_string()),
                    ignore_errors: None,
                    rollback_command: None,
                    interactive: Some(false),
                    inputs: None,
                    variables_file: None,
                },
            ],
            fail_fast: Some(true),
        }
    }

    /// Создает стандартное событие "post-deploy"
    pub fn create_post_deploy_event() -> config::Event {
        config::Event {
            name: POST_DEPLOY_NAME.to_string(),
            description: Some(POST_DEPLOY_DESC.to_string()),
            commands: vec![
                config::Command {
                    command: POST_DEPLOY_RESTART_CMD.to_string(),
                    description: Some(POST_DEPLOY_RESTART_DESC.to_string()),
                    ignore_errors: None,
                    rollback_command: Some(POST_DEPLOY_RESTART_ROLLBACK.to_string()),
                    interactive: Some(false),
                    inputs: None,
                    variables_file: None,
                },
                config::Command {
                    command: POST_DEPLOY_FINISH_CMD.to_string(),
                    description: Some(POST_DEPLOY_FINISH_DESC.to_string()),
                    ignore_errors: Some(true),
                    rollback_command: None,
                    interactive: Some(false),
                    inputs: None,
                    variables_file: None,
                },
            ],
            fail_fast: Some(false),
        }
    }
}

/// Получает конфигурацию деплоя по имени
pub fn get_deployment_config<'a>(
    config: &'a Config,
    deployment_name: &str,
) -> Result<&'a Deployment> {
    config
        .find_deployment(deployment_name)
        .with_context(|| format!("Деплой с именем '{}' не найден", deployment_name))
}

/// Проверяет, что все события в деплое имеют команды
pub fn validate_deployment_events(deployment: &Deployment) -> Result<bool> {
    // Проверка на пустые события
    if deployment.events.is_empty() {
        error!("Деплой '{}' не содержит событий", deployment.name);
        return Ok(false);
    }

    // Проверка каждого события на наличие команд
    for event in &deployment.events {
        if event.commands.is_empty() {
            error!(
                "Событие '{}' в деплое '{}' не содержит команд",
                event.name, deployment.name
            );
            return Ok(false);
        }
    }

    Ok(true)
}

// Реэкспортируем функции из шаблонов для обратной совместимости
pub use templates::create_new_deployment;
