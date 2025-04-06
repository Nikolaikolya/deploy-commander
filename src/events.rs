/*!
# Модуль Events

Модуль `events` предоставляет систему событий для процесса деплоя:

- Определение типов событий, происходящих в процессе деплоя
- Создание и отправка событий через эмиттер
- Асинхронная обработка событий через каналы

## Основные компоненты

- `EventType` - перечисление типов событий деплоя
- `EventEmitter` - компонент для отправки событий и логирования
*/

use log::{error, info};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// Типы событий, которые могут происходить во время деплоя
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    /// Начало деплоя
    DeploymentStarted {
        /// Имя деплоя
        deployment: String,
        /// Имя события
        event: String,
    },
    /// Успешное завершение деплоя
    DeploymentSucceeded {
        /// Имя деплоя
        deployment: String,
        /// Имя события
        event: String,
    },
    /// Ошибка деплоя
    DeploymentFailed {
        /// Имя деплоя
        deployment: String,
        /// Имя события
        event: String,
    },
    /// Ошибка команды
    CommandFailed {
        /// Имя деплоя
        deployment: String,
        /// Имя события
        event: String,
        /// Команда, вызвавшая ошибку
        command: String,
        /// Текст ошибки
        error: String,
    },
}

/// Эмиттер событий для отправки уведомлений о процессе деплоя
pub struct EventEmitter {
    /// Канал для отправки событий (опционально)
    sender: Option<mpsc::Sender<EventType>>,
}

impl EventEmitter {
    /// Создает новый эмиттер событий
    ///
    /// # Возвращаемое значение
    ///
    /// Экземпляр `EventEmitter` без настроенного канала
    pub fn new() -> Self {
        // В реальном приложении здесь можно было бы настроить отправку событий
        // в систему мониторинга, очередь сообщений и т.д.
        Self { sender: None }
    }

    /// Отправляет событие
    ///
    /// # Параметры
    ///
    /// * `event` - Событие для отправки и логирования
    ///
    /// # Примечания
    ///
    /// Метод логирует информацию о событии, а также отправляет его
    /// в канал, если он был настроен
    pub fn emit(&self, event: EventType) {
        match &event {
            EventType::DeploymentStarted { deployment, event } => {
                info!(
                    "Событие: Начало деплоя '{}', событие '{}'",
                    deployment, event
                );
            }
            EventType::DeploymentSucceeded { deployment, event } => {
                info!(
                    "Событие: Успешное завершение деплоя '{}', событие '{}'",
                    deployment, event
                );
            }
            EventType::DeploymentFailed { deployment, event } => {
                error!(
                    "Событие: Ошибка деплоя '{}', событие '{}'",
                    deployment, event
                );
            }
            EventType::CommandFailed {
                deployment,
                event,
                command,
                error,
            } => {
                error!(
                    "Событие: Ошибка команды '{}' в деплое '{}', событие '{}': {}",
                    command, deployment, event, error
                );
            }
        }

        // Если есть канал, отправляем событие
        if let Some(sender) = &self.sender {
            let sender = sender.clone();
            let event_clone = event.clone();

            tokio::spawn(async move {
                if let Err(e) = sender.send(event_clone).await {
                    eprintln!("Ошибка отправки события: {}", e);
                }
            });
        }
    }
}
