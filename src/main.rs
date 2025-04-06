use clap::Parser;
use log::{error, info, warn};
use std::process::exit;

mod cli;
mod commands;
mod config;
mod events;
mod executor;
mod logging;
mod storage;

use cli::Cli;
use config::Config;

/// Выполняет указанное событие для деплоя
async fn run_specific_event(config: &Config, deployment: &str, event: &str, history_path: &str) {
    info!(
        "Запуск команд для деплоя '{}', событие: '{}'",
        deployment, event
    );

    // Запись события начала деплоя
    if let Err(e) = storage::record_deployment(
        history_path,
        deployment,
        &format!("start-{}", event),
        true,
        None,
    ) {
        warn!("Ошибка записи события: {}", e);
    }

    match executor::run_commands(config, deployment, event).await {
        Ok(_) => {
            info!("Все команды выполнены успешно");
            // Запись успешного завершения уже выполняется в executor
        }
        Err(e) => {
            error!("Ошибка выполнения команд: {}", e);
            // Запись ошибки уже выполняется в executor
            exit(1);
        }
    }
}

#[tokio::main]
async fn main() {
    // Парсинг аргументов командной строки
    let cli = Cli::parse();

    // Настройка логирования
    if let Err(e) = logging::setup_logger(&cli.log_file, cli.verbose) {
        eprintln!("Ошибка настройки логирования: {}", e);
        exit(1);
    }

    info!("Запуск Deploy Commander v{}", env!("CARGO_PKG_VERSION"));

    // Проверка наличия необходимых команд
    if let Err(e) = commands::check_required_commands() {
        error!("Ошибка проверки команд: {}", e);
        exit(1);
    }

    // Загрузка конфигурации
    let config = match Config::load(&cli.config) {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Ошибка загрузки конфигурации: {}", e);
            exit(1);
        }
    };

    // Путь к файлу истории деплоев
    let history_path = "deploy-history.json";

    // Выполнение команды
    match cli.command {
        cli::Command::Run { deployment, event } => {
            // Если событие указано, запускаем только его
            if let Some(event_name) = event {
                run_specific_event(&config, &deployment, &event_name, history_path).await;
            } else {
                // Если событие не указано, запускаем все события последовательно
                info!("Запуск всех событий для деплоя '{}'", deployment);

                let deployment_config = match config.find_deployment(&deployment) {
                    Some(d) => d,
                    None => {
                        error!("Деплой с именем '{}' не найден", deployment);
                        exit(1);
                    }
                };

                // Запись события начала деплоя
                if let Err(e) = storage::record_deployment(
                    history_path,
                    &deployment,
                    "start-full-deploy",
                    true,
                    None,
                ) {
                    warn!("Ошибка записи события: {}", e);
                }

                let mut success = true;

                // Выполняем все события последовательно
                for event in &deployment_config.events {
                    info!(
                        "Запуск события '{}' для деплоя '{}'",
                        event.name, deployment
                    );

                    match executor::run_commands(&config, &deployment, &event.name).await {
                        Ok(_) => {
                            info!("Событие '{}' успешно выполнено", event.name);
                        }
                        Err(e) => {
                            error!("Ошибка выполнения события '{}': {}", event.name, e);
                            // Запись ошибки
                            if let Err(log_err) = storage::record_deployment(
                                history_path,
                                &deployment,
                                &format!("failed-{}", event.name),
                                false,
                                Some(e.to_string()),
                            ) {
                                warn!("Ошибка записи события: {}", log_err);
                            }
                            success = false;
                            break;
                        }
                    }
                }

                // Запись результата полного деплоя
                if success {
                    info!("Все события для деплоя '{}' успешно выполнены", deployment);
                    if let Err(e) = storage::record_deployment(
                        history_path,
                        &deployment,
                        "complete-full-deploy",
                        true,
                        Some("Все события деплоя успешно завершены".to_string()),
                    ) {
                        warn!("Ошибка записи события: {}", e);
                    }
                } else {
                    error!("Деплой '{}' завершился с ошибками", deployment);
                    if let Err(e) = storage::record_deployment(
                        history_path,
                        &deployment,
                        "failed-full-deploy",
                        false,
                        Some("Одно из событий завершилось с ошибкой".to_string()),
                    ) {
                        warn!("Ошибка записи события: {}", e);
                    }
                    exit(1);
                }
            }
        }
        cli::Command::List => {
            info!("Список доступных деплоев:");
            for deployment in config.deployments {
                println!("Деплой: {}", deployment.name);
                println!("  События:");
                for event in deployment.events {
                    println!("    {}", event.name);
                    println!("      Команды:");
                    for command in event.commands {
                        println!("        - {}", command.command);
                    }
                }
                println!();
            }
        }
        cli::Command::Create { deployment } => {
            info!("Создание шаблона деплоя: {}", deployment);
            match config::create_template_deployment(&deployment, &cli.config) {
                Ok(_) => info!("Шаблон деплоя '{}' успешно создан", deployment),
                Err(e) => {
                    error!("Ошибка создания шаблона: {}", e);
                    exit(1);
                }
            }
        }
        cli::Command::Verify { deployment } => {
            info!("Проверка конфигурации деплоя: {}", deployment);
            match config::verify_deployment(&config, &deployment) {
                Ok(true) => info!("Конфигурация деплоя '{}' корректна", deployment),
                Ok(false) => {
                    error!("Конфигурация деплоя '{}' некорректна", deployment);
                    exit(1);
                }
                Err(e) => {
                    error!("Ошибка проверки конфигурации: {}", e);
                    exit(1);
                }
            }
        }
        cli::Command::History { deployment, limit } => {
            info!("Просмотр истории деплоя: {}", deployment);

            match storage::DeploymentHistory::load(history_path) {
                Ok(history) => {
                    let records = history.get_records(&deployment, limit);

                    if records.is_empty() {
                        println!("История деплоя '{}' пуста", deployment);
                    } else {
                        println!(
                            "История деплоя '{}' (последние {} записей):",
                            deployment, limit
                        );
                        for (i, record) in records.iter().enumerate() {
                            let timestamp = chrono::DateTime::<chrono::Utc>::from(
                                std::time::UNIX_EPOCH
                                    + std::time::Duration::from_secs(record.timestamp),
                            )
                            .format("%Y-%m-%d %H:%M:%S");

                            let status = if record.success { "✅" } else { "❌" };

                            println!(
                                "{}. [{} UTC] {} {} {}",
                                i + 1,
                                timestamp,
                                status,
                                record.event,
                                record.details.as_deref().unwrap_or("")
                            );
                        }
                    }
                }
                Err(e) => {
                    error!("Ошибка загрузки истории деплоев: {}", e);
                    exit(1);
                }
            }
        }
        cli::Command::ClearHistory { deployment } => {
            match storage::clear_deployment_history(history_path, deployment.as_deref()) {
                Ok(_) => {
                    if let Some(dep) = deployment {
                        info!("История деплоя '{}' успешно очищена", dep);
                    } else {
                        info!("Вся история деплоев успешно очищена");
                    }
                }
                Err(e) => {
                    error!("Ошибка очистки истории: {}", e);
                    exit(1);
                }
            }
        }
    }

    info!("Работа Deploy Commander завершена");
}
