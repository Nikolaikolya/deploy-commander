/*!
# Модуль Run

Модуль `run` содержит основной функционал приложения Deploy Commander и реализует:

- Выполнение развертывания приложений по их конфигурации
- Запуск отдельных событий или всех событий в заданном порядке
- Обработку ошибок и откат изменений при необходимости
- Отображение и управление историей деплоев
- Параллельное выполнение всех деплоев для увеличения производительности

## Структура модуля

- `command_runner` - запуск команд и управление их выполнением
- `deployments` - работа с конфигурацией деплоев и их валидация
- `deployment_manager` - управление деплоями и их выполнением
- `deployment` - структура для работы с деплоями
- `history` - хранение и отображение истории деплоев

## Основные функции

- `run_event` - запускает отдельное событие для деплоя
- `run_all_events` - последовательно запускает все события для деплоя
- `list_deployments` - выводит список всех доступных деплоев
- `show_deployment_history` - отображает историю деплоев
- `run_all_deployments` - запускает все деплои параллельно или последовательно
*/

use crate::config::Config;
use log::info;

// Модули приложения
mod command_runner;
mod deployment;
mod deployment_manager;
pub mod deployments;
mod history;

pub use deployment::{create_deployment_template, Deployment};

/// Выполняет указанное событие для деплоя
///
/// # Параметры
///
/// * `config` - Конфигурация деплоя
/// * `deployment` - Имя деплоя
/// * `event` - Имя события
/// * `history_path` - Путь к файлу истории деплоев
pub async fn run_event(config: &Config, deployment: &str, event: &str, history_path: &str) {
    // Создаем экземпляр структуры Deployment
    let deployment_obj = Deployment::new(config.clone(), history_path.to_string(), false);
    deployment_obj.run_specific_event(deployment, event).await;
}

/// Запускает все события для указанного деплоя последовательно
///
/// # Параметры
///
/// * `config` - Конфигурация деплоя
/// * `deployment` - Имя деплоя
/// * `history_path` - Путь к файлу истории деплоев
pub async fn run_all_events(config: &Config, deployment: &str, history_path: &str) {
    // Создаем экземпляр структуры Deployment
    let deployment_obj = Deployment::new(config.clone(), history_path.to_string(), false);
    deployment_obj.run_all_events(deployment).await;
}

/// Запускает все доступные деплои в конфигурации, опционально с указанным событием
///
/// # Параметры
///
/// * `config` - Конфигурация деплоя
/// * `history_path` - Путь к файлу истории деплоев
/// * `event` - Опциональное имя события
/// * `parallel` - Флаг параллельного выполнения
pub async fn run_all_deployments(
    config: &Config,
    history_path: &str,
    event: Option<&str>,
    parallel: bool,
) {
    info!(
        "Запуск всех доступных деплоев в режиме {}",
        if parallel {
            "параллельном"
        } else {
            "последовательном"
        }
    );

    // Создаем экземпляр структуры Deployment
    let deployment_obj = Deployment::new(config.clone(), history_path.to_string(), parallel);
    deployment_obj.run_all_deployments(event, parallel).await;
}

/// Выводит список всех доступных деплоев и команд
///
/// # Параметры
///
/// * `config` - Конфигурация деплоя
pub fn list_deployments(config: &Config) {
    // Создаем экземпляр структуры Deployment
    let deployment_obj = Deployment::new(config.clone(), String::new(), false);
    deployment_obj.list_deployments();
}

/// Проверяет конфигурацию деплоя на корректность
///
/// # Параметры
///
/// * `config` - Конфигурация деплоя
/// * `deployment_name` - Имя деплоя для проверки
pub fn verify_deployment_config(config: &Config, deployment_name: &str) {
    // Создаем экземпляр структуры Deployment
    let deployment_obj = Deployment::new(config.clone(), String::new(), false);
    deployment_obj.verify_deployment_config(deployment_name);
}

/// Показывает историю деплоев
///
/// # Параметры
///
/// * `history_path` - Путь к файлу истории деплоев
/// * `deployment_name` - Имя деплоя
/// * `limit` - Количество записей истории для отображения
pub fn show_deployment_history(history_path: &str, deployment_name: &str, limit: usize) {
    // Создаем экземпляр структуры Deployment
    let deployment_obj = Deployment::new(Config::default(), history_path.to_string(), false);
    deployment_obj.show_deployment_history(deployment_name, limit);
}

/// Очищает историю деплоев
///
/// # Параметры
///
/// * `history_path` - Путь к файлу истории деплоев
/// * `deployment_name` - Имя деплоя (если None, очищается вся история)
pub fn clear_deployment_history(history_path: &str, deployment_name: Option<&str>) {
    // Создаем экземпляр структуры Deployment
    let deployment_obj = Deployment::new(Config::default(), history_path.to_string(), false);
    deployment_obj.clear_deployment_history(deployment_name);
}
