use clap::Parser;
use log::{error, info};
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

    // Загрузка конфигурации
    let config = match Config::load(&cli.config) {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Ошибка загрузки конфигурации: {}", e);
            exit(1);
        }
    };

    // Выполнение команды
    match cli.command {
        cli::Command::Run { deployment, event } => {
            info!("Запуск команд для деплоя '{}', событие: '{}'", deployment, event);
            match executor::run_commands(&config, &deployment, &event).await {
                Ok(_) => info!("Все команды выполнены успешно"),
                Err(e) => {
                    error!("Ошибка выполнения команд: {}", e);
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
    }

    info!("Работа Deploy Commander завершена");
}
