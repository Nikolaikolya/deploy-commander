use log::{debug, info, trace, warn};
use std::time::Instant;

use crate::cli::Cli;
use crate::commands;
use crate::config::Config;
use crate::logging;
use crate::run;
use crate::settings::{get_settings, Settings, DEFAULT_SETTINGS_PATH};

/// Глобальные настройки приложения
#[derive(Debug)]
pub struct AppContext {
    /// Глобальные настройки приложения
    pub settings: Settings,

    /// Конфигурация деплоя
    pub config: Config,

    /// Режим выполнения (параллельный или последовательный)
    pub parallel_execution: bool,
}

/// Инициализирует приложение и настраивает логирование
pub fn initialize(cli: &Cli) -> Result<AppContext, String> {
    // Загрузка настроек
    let settings = match get_settings(DEFAULT_SETTINGS_PATH) {
        Ok(settings) => settings,
        Err(e) => return Err(format!("Ошибка загрузки настроек: {}", e)),
    };

    // Настройка логирования
    if let Err(e) = logging::setup_logger(&settings.log_file, cli.verbose) {
        return Err(format!("Ошибка настройки логирования: {}", e));
    }

    info!("Запуск Deploy Commander v{}", env!("CARGO_PKG_VERSION"));

    // Загрузка конфигурации деплоя
    let config = match load_config(&cli.config) {
        Ok(cfg) => cfg,
        Err(e) => return Err(e),
    };

    // Определяем режим выполнения (параллельный по умолчанию)
    let parallel_execution = cli.parallel.unwrap_or(true);
    info!(
        "Режим выполнения: {}",
        if parallel_execution {
            "параллельный"
        } else {
            "последовательный"
        }
    );

    Ok(AppContext {
        settings,
        config,
        parallel_execution,
    })
}

/// Проверяет наличие необходимых внешних команд
pub async fn check_dependencies() {
    trace!("Проверка наличия необходимых внешних команд");
    let start_time = Instant::now();

    if let Err(e) = commands::check_required_commands().await {
        warn!("Некоторые команды недоступны: {}", e);
    }

    let duration = start_time.elapsed();
    debug!(
        "Проверка зависимостей завершена за {:.2} мс",
        duration.as_millis()
    );
}

/// Загружает конфигурацию
pub fn load_config(config_path: &str) -> Result<Config, String> {
    info!("Загрузка конфигурации из файла: {}", config_path);
    let start_time = Instant::now();

    let result = match Config::load(config_path) {
        Ok(cfg) => {
            let duration = start_time.elapsed();
            info!(
                "Конфигурация успешно загружена за {:.2} мс, содержит {} деплоев",
                duration.as_millis(),
                cfg.deployments.len()
            );
            Ok(cfg)
        }
        Err(e) => Err(format!("Ошибка загрузки конфигурации: {}", e)),
    };

    result
}

/// Запускает указанный деплой с конкретным событием
///
/// # Параметры
///
/// * `app_context` - Контекст приложения
/// * `deployment` - Имя деплоя
/// * `event` - Опциональное имя события
async fn handle_run_command(app_context: &AppContext, deployment: &str, event: &Option<String>) {
    let history_path = &app_context.settings.history_file;

    trace!(
        "Запуск команды для деплоя '{}', событие: {:?}",
        deployment,
        event
    );

    // Проверяем на специальное значение "all"
    if deployment == "all" {
        info!("Запуск всех доступных деплоев из конфигурации");
        // Передаем опцию parallel_execution для определения режима выполнения
        run::run_all_deployments(
            &app_context.config,
            history_path,
            event.as_deref(),
            app_context.parallel_execution,
        )
        .await;
    } else {
        // Если событие указано, запускаем только его
        if let Some(event_name) = event {
            info!("Запуск деплоя '{}', событие '{}'", deployment, event_name);
            run::run_event(&app_context.config, deployment, event_name, history_path).await;
        } else {
            // Если событие не указано, запускаем все события последовательно
            info!("Запуск всех событий для деплоя '{}'", deployment);
            run::run_all_events(&app_context.config, deployment, history_path).await;
        }
    }
}

/// Отображает список всех доступных деплоев
///
/// # Параметры
///
/// * `app_context` - Контекст приложения
fn handle_list_command(app_context: &AppContext) {
    info!("Отображение списка доступных деплоев");
    run::list_deployments(&app_context.config);
}

/// Создает шаблон деплоя
///
/// # Параметры
///
/// * `deployment` - Имя деплоя
/// * `config_path` - Путь к конфигурационному файлу
fn handle_create_command(deployment: &str, config_path: &str) {
    info!("Создание шаблона деплоя '{}'", deployment);
    run::create_deployment_template(deployment, config_path);
}

/// Проверяет конфигурацию деплоя
///
/// # Параметры
///
/// * `app_context` - Контекст приложения
/// * `deployment` - Имя деплоя
fn handle_verify_command(app_context: &AppContext, deployment: &str) {
    info!("Проверка конфигурации деплоя '{}'", deployment);
    run::verify_deployment_config(&app_context.config, deployment);
}

/// Отображает историю деплоя
///
/// # Параметры
///
/// * `app_context` - Контекст приложения
/// * `deployment` - Имя деплоя
/// * `limit` - Количество записей для отображения
fn handle_history_command(app_context: &AppContext, deployment: &str, limit: usize) {
    info!(
        "Отображение истории деплоя '{}' (лимит: {})",
        deployment, limit
    );
    run::show_deployment_history(&app_context.settings.history_file, deployment, limit);
}

/// Очищает историю деплоев
///
/// # Параметры
///
/// * `app_context` - Контекст приложения
/// * `deployment` - Опциональное имя деплоя
fn handle_clear_history_command(app_context: &AppContext, deployment: &Option<String>) {
    if let Some(dep) = deployment {
        info!("Очистка истории деплоя '{}'", dep);
    } else {
        info!("Очистка всей истории деплоев");
    }
    run::clear_deployment_history(&app_context.settings.history_file, deployment.as_deref());
}

/// Выполняет команду в зависимости от аргументов командной строки
pub async fn execute_command(cli: &Cli, app_context: &AppContext) {
    let start_time = Instant::now();

    debug!("Начало выполнения команды: {:?}", cli.command);

    match &cli.command {
        crate::cli::Command::Run { deployment, event } => {
            handle_run_command(app_context, deployment, event).await;
        }
        crate::cli::Command::List => {
            handle_list_command(app_context);
        }
        crate::cli::Command::Create { deployment } => {
            handle_create_command(deployment, &cli.config);
        }
        crate::cli::Command::Verify { deployment } => {
            handle_verify_command(app_context, deployment);
        }
        crate::cli::Command::History { deployment, limit } => {
            handle_history_command(app_context, deployment, *limit);
        }
        crate::cli::Command::ClearHistory { deployment } => {
            handle_clear_history_command(app_context, deployment);
        }
    }

    let duration = start_time.elapsed();
    info!(
        "Работа Deploy Commander завершена за {:.2} секунд",
        duration.as_secs_f64()
    );
}
