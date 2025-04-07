/*!
# Модуль DeploymentManager

Отвечает за управление деплоями и их выполнение:

- Управление жизненным циклом деплоя
- Запуск деплоев в разных режимах (параллельно/последовательно)
- Обработка ошибок и журналирование
- Управление состоянием деплоя
*/

use anyhow::{Context, Result};
use log::{error, info, warn};
use std::sync::{Arc, Mutex};

use crate::config::{Config, Deployment};
use crate::run::command_runner;
use crate::storage;

/// Структура для управления деплоем
pub struct DeploymentManager<'a> {
    /// Ссылка на конфигурацию деплоев
    config: &'a Config,

    /// Путь к файлу истории деплоев
    history_path: &'a str,

    /// Список неудачных деплоев
    failed_deployments: Arc<Mutex<Vec<String>>>,
}

impl<'a> DeploymentManager<'a> {
    /// Создает новый экземпляр менеджера деплоев
    ///
    /// # Параметры
    ///
    /// * `config` - Конфигурация деплоя
    /// * `history_path` - Путь к файлу истории деплоев
    /// * `parallel_execution` - Флаг параллельного выполнения (не используется, оставлен для обратной совместимости)
    pub fn new(config: &'a Config, history_path: &'a str, _parallel_execution: bool) -> Self {
        Self {
            config,
            history_path,
            failed_deployments: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Получает деплой по имени
    ///
    /// # Параметры
    ///
    /// * `deployment_name` - Имя деплоя
    ///
    /// # Возвращаемое значение
    ///
    /// Результат с ссылкой на деплой или ошибку
    pub fn get_deployment(&self, deployment_name: &str) -> Result<&Deployment> {
        self.config
            .find_deployment(deployment_name)
            .with_context(|| format!("Деплой с именем '{}' не найден", deployment_name))
    }

    /// Записывает событие о начале деплоя
    ///
    /// # Параметры
    ///
    /// * `deployment_name` - Имя деплоя
    /// * `event_type` - Тип события
    fn record_start(&self, deployment_name: &str, event_type: &str) {
        if let Err(e) =
            storage::record_deployment(self.history_path, deployment_name, event_type, true, None)
        {
            warn!("Ошибка записи события: {}", e);
        }
    }

    /// Записывает успешное завершение деплоя
    ///
    /// # Параметры
    ///
    /// * `deployment_name` - Имя деплоя
    /// * `event_type` - Тип события
    /// * `message` - Опциональное сообщение
    fn record_success(&self, deployment_name: &str, event_type: &str, message: Option<String>) {
        if let Err(e) =
            command_runner::record_success(self.history_path, deployment_name, event_type, message)
        {
            warn!("Ошибка записи успешного события: {}", e);
        }
    }

    /// Записывает сбой деплоя
    ///
    /// # Параметры
    ///
    /// * `deployment_name` - Имя деплоя
    /// * `event_type` - Тип события
    /// * `error_message` - Сообщение об ошибке
    fn record_failure(&self, deployment_name: &str, event_type: &str, error_message: String) {
        if let Err(e) = command_runner::record_failure(
            self.history_path,
            deployment_name,
            event_type,
            error_message,
        ) {
            warn!("Ошибка записи события сбоя: {}", e);
        }
    }

    /// Выполняет указанное событие для деплоя
    ///
    /// # Параметры
    ///
    /// * `deployment_name` - Имя деплоя
    /// * `event_name` - Имя события
    ///
    /// # Возвращаемое значение
    ///
    /// Результат выполнения (true - успешно, false - с ошибками)
    pub async fn execute_event(&self, deployment_name: &str, event_name: &str) -> bool {
        info!(
            "Запуск события '{}' для деплоя '{}'",
            event_name, deployment_name
        );

        match command_runner::execute_command(
            self.config,
            deployment_name,
            event_name,
            self.history_path,
        )
        .await
        {
            Ok(_) => {
                info!(
                    "Деплой '{}', событие '{}' успешно выполнено",
                    deployment_name, event_name
                );
                true
            }
            Err(e) => {
                error!(
                    "Ошибка выполнения деплоя '{}', событие '{}': {}",
                    deployment_name, event_name, e
                );

                // Запись ошибки
                self.record_failure(
                    deployment_name,
                    &format!("failed-{}", event_name),
                    e.to_string(),
                );

                // Добавляем в список неудачных деплоев
                let mut failed = self.failed_deployments.lock().unwrap();
                failed.push(deployment_name.to_string());

                false
            }
        }
    }

    /// Выполняет все события для деплоя
    ///
    /// # Параметры
    ///
    /// * `deployment_name` - Имя деплоя
    ///
    /// # Возвращаемое значение
    ///
    /// Результат выполнения (true - успешно, false - с ошибками)
    pub async fn execute_all_events(&self, deployment_name: &str) -> bool {
        info!("Запуск всех событий для деплоя '{}'", deployment_name);

        let deployment = match self.get_deployment(deployment_name) {
            Ok(d) => d,
            Err(e) => {
                error!(
                    "Ошибка получения конфигурации деплоя '{}': {}",
                    deployment_name, e
                );
                let mut failed = self.failed_deployments.lock().unwrap();
                failed.push(deployment_name.to_string());
                return false;
            }
        };

        // Запись события начала деплоя
        self.record_start(deployment_name, "start-full-deploy");

        let mut success = true;

        // Выполняем все события последовательно
        for event in &deployment.events {
            // Если предыдущее событие не удалось, прерываем выполнение
            if !success {
                break;
            }

            // Выполняем событие
            success = self.execute_event(deployment_name, &event.name).await;
        }

        // Запись итогового результата
        if success {
            info!(
                "Все события для деплоя '{}' успешно выполнены",
                deployment_name
            );
            self.record_success(
                deployment_name,
                "complete-full-deploy",
                Some("Все события деплоя успешно завершены".to_string()),
            );
        } else {
            error!("Деплой '{}' завершился с ошибками", deployment_name);
            self.record_failure(
                deployment_name,
                "failed-full-deploy",
                "Одно из событий завершилось с ошибкой".to_string(),
            );
        }

        success
    }

    /// Отображает список всех деплоев с их событиями и командами
    pub fn list_deployments(&self) {
        info!("Список доступных деплоев:");
        for deployment in &self.config.deployments {
            println!("Деплой: {}", deployment.name);
            println!("  События:");
            for event in &deployment.events {
                println!("    {}", event.name);
                println!("      Команды:");
                for command in &event.commands {
                    println!("        - {}", command.command);
                }
            }
            println!();
        }
    }
}
