/*!
# Модуль Run

Модуль `run` содержит основной функционал приложения Deploy Commander.
*/

// Подмодули
mod command_runner;
mod deployment;
mod deployment_manager;
pub mod deployments;
mod history;

// Реэкспорт публичных типов и функций из модуля deployment
pub use deployment::{create_deployment_template, Deployment};

// Создаем публичные функции-обертки для методов структуры Deployment
use crate::config::Config;

/// Выполняет указанное событие для деплоя
pub async fn run_event(config: &Config, deployment: &str, event: &str, history_path: &str) {
    let deployment_obj = Deployment::new(config.clone(), history_path.to_string(), false);
    deployment_obj.run_specific_event(deployment, event).await;
}

/// Запускает все события для указанного деплоя последовательно
pub async fn run_all_events(config: &Config, deployment: &str, history_path: &str) {
    let deployment_obj = Deployment::new(config.clone(), history_path.to_string(), false);
    deployment_obj.run_all_events(deployment).await;
}

/// Запускает все доступные деплои в конфигурации
pub async fn run_all_deployments(
    config: &Config,
    history_path: &str,
    event: Option<&str>,
    parallel: bool,
) {
    let deployment_obj = Deployment::new(config.clone(), history_path.to_string(), parallel);
    deployment_obj.run_all_deployments(event, parallel).await;
}

/// Выводит список всех доступных деплоев и команд
pub fn list_deployments(config: &Config) {
    let deployment_obj = Deployment::new(config.clone(), String::new(), false);
    deployment_obj.list_deployments();
}

/// Проверяет конфигурацию деплоя на корректность
pub fn verify_deployment_config(config: &Config, deployment_name: &str) {
    let deployment_obj = Deployment::new(config.clone(), String::new(), false);
    deployment_obj.verify_deployment_config(deployment_name);
}

/// Показывает историю деплоев
pub fn show_deployment_history(history_path: &str, deployment_name: &str, limit: usize) {
    let deployment_obj = Deployment::new(Config::default(), history_path.to_string(), false);
    deployment_obj.show_deployment_history(deployment_name, limit);
}

/// Очищает историю деплоев
pub fn clear_deployment_history(history_path: &str, deployment_name: Option<&str>) {
    let deployment_obj = Deployment::new(Config::default(), history_path.to_string(), false);
    deployment_obj.clear_deployment_history(deployment_name);
}
