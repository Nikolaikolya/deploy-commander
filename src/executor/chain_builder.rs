/*!
# Подмодуль Chain Builder

Отвечает за создание и настройку цепочек команд:

- Создание цепочек с правильными параметрами
- Добавление команд в цепочку с оптимизацией производительности
- Настройка режима выполнения цепочек (последовательный или параллельный)
- Поддержка переменных из разных источников с приоритезацией
- Интеграция с глобальными переменными из settings.json
- Мониторинг и логирование процесса построения цепочек
*/

use crate::config::Config;
use crate::executor::command_executor;
use anyhow::{Context, Result};
use command_system::{ChainBuilder, ChainExecutionMode, ConsoleLogger, LogLevel};
use log::{debug, info, trace};
use std::path::Path;
use std::time::Instant;

/// Находит деплойную конфигурацию и событие по имени
///
/// # Параметры
///
/// * `config` - Конфигурация деплоя
/// * `deployment_name` - Имя деплоя
/// * `event_name` - Имя события
///
/// # Возвращаемое значение
///
/// Кортеж из ссылки на деплойную конфигурацию и событие
fn find_deployment_and_event<'a>(
    config: &'a Config,
    deployment_name: &str,
    event_name: &str,
) -> Result<(&'a crate::config::Deployment, &'a crate::config::Event)> {
    // Находим деплой и событие
    let deployment = config
        .find_deployment(deployment_name)
        .with_context(|| format!("Деплой '{}' не найден", deployment_name))?;

    let event = deployment
        .events
        .iter()
        .find(|e| e.name == event_name)
        .with_context(|| {
            format!(
                "Событие '{}' не найдено в деплое '{}'",
                event_name, deployment_name
            )
        })?;

    debug!(
        "Найдено событие '{}' в деплое '{}' с {} командами",
        event_name,
        deployment_name,
        event.commands.len()
    );

    Ok((deployment, event))
}

/// Определяет переменные окружения для деплоя
///
/// # Параметры
///
/// * `deployment` - Конфигурация деплоя
/// * `deployment_name` - Имя деплоя
///
/// # Возвращаемое значение
///
/// Вектор пар (имя_переменной, значение_переменной)
fn determine_environment_variables(
    deployment: &crate::config::Deployment,
    deployment_name: &str,
) -> Vec<(String, String)> {
    deployment
        .environment
        .as_ref()
        .map(|vars| {
            let env_vars: Vec<_> = vars
                .iter()
                .filter_map(|var| {
                    let parts: Vec<&str> = var.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        Some((parts[0].to_string(), parts[1].to_string()))
                    } else {
                        None
                    }
                })
                .collect();

            debug!(
                "Для деплоя '{}' определено {} переменных окружения",
                deployment_name,
                env_vars.len()
            );

            env_vars
        })
        .unwrap_or_default()
}

/// Логирует информацию о переменных
///
/// # Параметры
///
/// * `variables_file` - Локальный файл переменных
/// * `global_variables_file` - Глобальный файл переменных
fn log_variables_info(variables_file: Option<&str>, global_variables_file: Option<&str>) {
    if let Some(file) = variables_file {
        if Path::new(file).exists() {
            info!("Используется локальный файл переменных: {}", file);
        } else {
            info!("Локальный файл переменных '{}' не найден", file);
        }
    }

    if let Some(global_file) = global_variables_file {
        if Path::new(global_file).exists() {
            info!("Используется глобальный файл переменных: {}", global_file);
        } else {
            info!("Глобальный файл переменных '{}' не найден", global_file);
        }
    }
}

/// Определяет режим выполнения цепочки
///
/// # Параметры
///
/// * `event` - Конфигурация события
///
/// # Возвращаемое значение
///
/// Режим выполнения цепочки (последовательный или параллельный)
fn determine_chain_execution_mode(event: &crate::config::Event) -> ChainExecutionMode {
    if event.fail_fast.unwrap_or(true) {
        info!("Режим выполнения цепочки: Sequential (fail-fast)");
        ChainExecutionMode::Sequential
    } else {
        info!("Режим выполнения цепочки: Parallel");
        ChainExecutionMode::Parallel
    }
}

/// Создает и настраивает новую цепочку команд
///
/// # Параметры
///
/// * `chain_name` - Имя цепочки
/// * `chain_mode` - Режим выполнения цепочки
///
/// # Возвращаемое значение
///
/// Настроенная цепочка команд
fn create_command_chain(
    chain_name: &str,
    chain_mode: ChainExecutionMode,
) -> command_system::chain::CommandChain {
    // Создаем логгер
    let logger = Box::new(ConsoleLogger::new(LogLevel::Info));

    // Создаем цепочку команд
    let chain = ChainBuilder::new(chain_name)
        .execution_mode(chain_mode)
        .logger(logger)
        .rollback_on_error(true) // Включаем откат при ошибке
        .build();

    info!(
        "Начало выполнения цепочки '{}' в режиме {}",
        chain_name,
        if chain_mode == ChainExecutionMode::Sequential {
            "Sequential (последовательный)"
        } else {
            "Parallel (параллельный)"
        }
    );

    chain
}

/// Добавляет команды в цепочку
///
/// # Параметры
///
/// * `chain` - Цепочка команд
/// * `deployment_name` - Имя деплоя
/// * `event_name` - Имя события
/// * `event` - Конфигурация события
/// * `working_dir` - Рабочая директория
/// * `env_vars` - Переменные окружения
/// * `variables_file` - Локальный файл переменных
/// * `global_variables_file` - Глобальный файл переменных
/// * `chain_name` - Имя цепочки команд
///
/// # Возвращаемое значение
///
/// Цепочка команд с добавленными командами и статистика
fn add_commands_to_chain(
    mut chain: command_system::chain::CommandChain,
    deployment_name: &str,
    event_name: &str,
    event: &crate::config::Event,
    working_dir: Option<&str>,
    env_vars: &[(String, String)],
    variables_file: Option<&str>,
    global_variables_file: Option<&str>,
    chain_name: &str,
) -> (command_system::chain::CommandChain, CommandStats) {
    // Подсчет команд с различными параметрами для информации
    let mut stats = CommandStats::default();

    // Собираем команды в цепочку с учетом рабочей директорий и переменных окружения
    for (idx, cmd) in event.commands.iter().enumerate() {
        let cmd_name = format!("{}_{}_cmd_{}", deployment_name, event_name, idx + 1);
        trace!(
            "Добавление команды '{}' в цепочку: {}",
            cmd_name,
            cmd.command
        );

        let ignore_errors = cmd.ignore_errors.unwrap_or(false);
        if ignore_errors {
            stats.commands_ignoring_errors += 1;
        }

        // Определяем команду отката
        let rollback_cmd = if !ignore_errors {
            let rollback = cmd.rollback_command.as_deref();
            if rollback.is_some() {
                stats.commands_with_rollback += 1;
            }
            rollback
        } else {
            None
        };

        // Проверяем, содержит ли команда шаблонные переменные
        let has_variables = cmd.command.contains('{') && cmd.command.contains('}');
        if has_variables {
            stats.commands_with_variables += 1;
        }

        // Проверяем, есть ли у команды свой файл с переменными
        let cmd_variables_file = cmd.variables_file.as_deref().or(variables_file);

        // Создаем команду с учетом переменных
        let command = if has_variables || cmd.interactive.unwrap_or(false) {
            // Используем переменные, если они указаны
            command_executor::create_command(
                &cmd_name,
                &cmd.command,
                working_dir,
                env_vars,
                rollback_cmd,
                true,
                cmd.inputs.clone(),
                cmd_variables_file,
                global_variables_file,
            )
        } else {
            // Для обычных команд используем простое создание
            command_executor::create_simple_command(
                &cmd_name,
                &cmd.command,
                working_dir,
                env_vars,
                rollback_cmd,
            )
        };

        // Логируем информацию о команде
        log_command_details(
            cmd,
            rollback_cmd,
            has_variables,
            cmd_variables_file,
            global_variables_file,
        );

        chain.add_command(command);
        info!(
            "Добавлена команда '{}' в цепочку '{}'",
            cmd_name, chain_name
        );
    }

    (chain, stats)
}

/// Логирует детали о создаваемой команде
///
/// # Параметры
///
/// * `cmd` - Конфигурация команды
/// * `rollback_cmd` - Команда отката
/// * `has_variables` - Флаг наличия переменных
/// * `cmd_variables_file` - Локальный файл переменных
/// * `global_variables_file` - Глобальный файл переменных
fn log_command_details(
    cmd: &crate::config::Command,
    rollback_cmd: Option<&str>,
    has_variables: bool,
    cmd_variables_file: Option<&str>,
    global_variables_file: Option<&str>,
) {
    if cmd.ignore_errors.unwrap_or(false) {
        debug!("Команда '{}' настроена игнорировать ошибки", cmd.command);
    } else if let Some(rollback) = rollback_cmd {
        debug!(
            "Команда '{}' настроена с откатом: {}",
            cmd.command, rollback
        );
    }

    if has_variables {
        debug!("Команда '{}' использует шаблонные переменные", cmd.command);
    }

    if let Some(file_path) = cmd_variables_file {
        if Path::new(file_path).exists() {
            debug!(
                "Команда '{}' может использовать переменные из файла: {}",
                cmd.command, file_path
            );
        } else {
            debug!(
                "Файл с переменными '{}' не найден, переменные будут запрошены интерактивно",
                file_path
            );
        }
    }

    if let Some(global_path) = global_variables_file {
        if Path::new(global_path).exists() {
            debug!(
                "Команда '{}' может использовать глобальные переменные из файла: {}",
                cmd.command, global_path
            );
        }
    }
}

/// Структура для хранения статистики команд
#[derive(Default)]
struct CommandStats {
    commands_with_rollback: usize,
    commands_with_variables: usize,
    commands_ignoring_errors: usize,
}

/// Строит цепочку команд для указанного деплоя и события
///
/// # Параметры
///
/// * `config` - Конфигурация деплоя
/// * `deployment_name` - Имя деплоя
/// * `event_name` - Имя события
/// * `global_variables_file` - Опциональный путь к глобальному файлу с переменными
///
/// # Возвращаемое значение
///
/// Настроенная цепочка команд, готовая к выполнению
pub fn build_command_chain(
    config: &Config,
    deployment_name: &str,
    event_name: &str,
    global_variables_file: Option<&str>,
) -> Result<command_system::chain::CommandChain> {
    // Засекаем время для метрик производительности
    let start_time = Instant::now();
    trace!(
        "Начало построения цепочки команд для деплоя '{}', событие '{}'",
        deployment_name,
        event_name
    );

    // Находим деплой и событие
    let (deployment, event) = find_deployment_and_event(config, deployment_name, event_name)?;

    // Определяем рабочую директорию
    let working_dir = deployment.working_dir.as_deref();
    if let Some(dir) = working_dir {
        debug!(
            "Рабочая директория для деплоя '{}': {}",
            deployment_name, dir
        );
    } else {
        debug!(
            "Для деплоя '{}' не задана рабочая директория, используется текущая",
            deployment_name
        );
    }

    // Определяем переменные окружения
    let env_vars = determine_environment_variables(deployment, deployment_name);

    // Определяем файл с переменными, если указан в деплойменте
    let variables_file = deployment.variables_file.as_deref();

    // Логируем информацию о переменных
    log_variables_info(variables_file, global_variables_file);

    // Создаем режим выполнения цепочки
    let chain_mode = determine_chain_execution_mode(event);

    // Создаем цепочку команд
    let chain_name = format!("{}_{}_chain", deployment_name, event_name);
    let chain = create_command_chain(&chain_name, chain_mode);

    // Добавляем команды в цепочку
    let (chain, stats) = add_commands_to_chain(
        chain,
        deployment_name,
        event_name,
        event,
        working_dir,
        &env_vars,
        variables_file,
        global_variables_file,
        &chain_name,
    );

    let duration = start_time.elapsed();
    info!(
        "Цепочка '{}' построена за {:.2} мс: {} команд, {} с откатом, {} с переменными, {} игнорируют ошибки",
        chain_name,
        duration.as_millis(),
        event.commands.len(),
        stats.commands_with_rollback,
        stats.commands_with_variables,
        stats.commands_ignoring_errors
    );

    Ok(chain)
}
