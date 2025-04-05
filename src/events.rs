use log::info;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
	DeploymentStarted {
		deployment: String,
		event: String,
	},
	DeploymentSucceeded {
		deployment: String,
		event: String,
	},
	DeploymentFailed {
		deployment: String,
		event: String,
	},
	CommandStarted {
		deployment: String,
		event: String,
		command: String,
		index: usize,
		total: usize,
	},
	CommandSucceeded {
		deployment: String,
		event: String,
		command: String,
		output: String,
	},
	CommandFailed {
		deployment: String,
		event: String,
		command: String,
		error: String,
	},
}

#[derive(Clone)]
pub struct EventEmitter {
	sender: Option<mpsc::Sender<EventType>>,
}

impl EventEmitter {
	pub fn new() -> Self {
		// В реальном приложении здесь можно было бы настроить отправку событий
		// в систему мониторинга, очередь сообщений и т.д.
		// Сейчас просто логируем события
		Self { sender: None }
	}

	pub fn with_channel(sender: mpsc::Sender<EventType>) -> Self {
		Self { sender: Some(sender) }
	}

	pub fn emit(&self, event: EventType) {
		// Логируем событие
		match &event {
			EventType::DeploymentStarted { deployment, event } => {
				info!("Событие: Начало деплоя '{}', событие '{}'", deployment, event);
			}
			EventType::DeploymentSucceeded { deployment, event } => {
				info!("Событие: Успешное завершение деплоя '{}', событие '{}'", deployment, event);
			}
			EventType::DeploymentFailed { deployment, event } => {
				info!("Событие: Ошибка деплоя '{}', событие '{}'", deployment, event);
			}
			EventType::CommandStarted { deployment, event, command, index, total } => {
				info!("Событие: Начало выполнения команды [{}/{}] '{}' для деплоя '{}', событие '{}'",
                     index, total, command, deployment, event);
			}
			EventType::CommandSucceeded { deployment, event, command, .. } => {
				info!("Событие: Успешное выполнение команды '{}' для деплоя '{}', событие '{}'",
                     command, deployment, event);
			}
			EventType::CommandFailed { deployment, event, command, error } => {
				info!("Событие: Ошибка выполнения команды '{}' для деплоя '{}', событие '{}': {}",
                     command, deployment, event, error);
			}
		}

		// Отправляем событие в канал, если он настроен
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

// Обработчик событий - может быть расширен для подключения
// внешних систем мониторинга, уведомлений и т.д.
pub struct EventHandler {
	receiver: mpsc::Receiver<EventType>,
}

impl EventHandler {
	pub fn new(receiver: mpsc::Receiver<EventType>) -> Self {
		Self { receiver }
	}

	pub async fn start(mut self) {
		while let Some(event) = self.receiver.recv().await {
			// Здесь можно добавить обработку событий, например:
			// - отправку уведомлений в Telegram
			// - запись в базу данных
			// - интеграцию с системами мониторинга
			// и т.д.

			match event {
				EventType::DeploymentStarted { .. } => {
					// Например, отправить уведомление о начале деплоя
				}
				EventType::DeploymentFailed { .. } => {
					// Отправить уведомление об ошибке
				}
				_ => {}
			}
		}
	}
}

// Создание пары каналов для событий
pub fn create_event_channels() -> (EventEmitter, EventHandler) {
	let (sender, receiver) = mpsc::channel(100);

	let emitter = EventEmitter::with_channel(sender);
	let handler = EventHandler::new(receiver);

	(emitter, handler)
}