use anyhow::Result;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeploymentHistory {
	pub deployments: HashMap<String, Vec<DeploymentRecord>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeploymentRecord {
	pub timestamp: u64,
	pub deployment: String,
	pub event: String,
	pub success: bool,
	pub details: Option<String>,
}

impl DeploymentHistory {
	pub fn load(path: &str) -> Result<Self> {
		let history_path = Path::new(path);

		if !history_path.exists() {
			info!("История деплоев не найдена, создаем новую: {}", path);
			return Ok(DeploymentHistory {
				deployments: HashMap::new(),
			});
		}

		let content = fs::read_to_string(history_path)?;
		Ok(serde_json::from_str(&content)?)
	}

	pub fn save(&self, path: &str) -> Result<()> {
		let content = serde_json::to_string_pretty(self)?;
		fs::write(path, content)?;
		Ok(())
	}

	pub fn add_record(&mut self, record: DeploymentRecord) {
		let records = self.deployments
			.entry(record.deployment.clone())
			.or_insert_with(Vec::new);

		// Ограничиваем историю 100 записями для каждого деплоя
		if records.len() >= 100 {
			records.remove(0);
		}

		records.push(record);
	}

	pub fn get_records(&self, deployment: &str, limit: usize) -> Vec<&DeploymentRecord> {
		self.deployments
			.get(deployment)
			.map(|records| {
				records.iter()
					.rev()
					.take(limit)
					.collect()
			})
			.unwrap_or_default()
	}
}

pub fn record_deployment(
	path: &str,
	deployment: &str,
	event: &str,
	success: bool,
	details: Option<String>,
) -> Result<()> {
	let mut history = match DeploymentHistory::load(path) {
		Ok(h) => h,
		Err(e) => {
			error!("Ошибка загрузки истории деплоев: {}", e);
			DeploymentHistory {
				deployments: HashMap::new(),
			}
		}
	};

	// Создаем новую запись
	let timestamp = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.expect("Ошибка получения времени")
		.as_secs();

	let record = DeploymentRecord {
		timestamp,
		deployment: deployment.to_string(),
		event: event.to_string(),
		success,
		details,
	};

	// Добавляем запись и сохраняем историю
	history.add_record(record);
	history.save(path)?;

	Ok(())
}
