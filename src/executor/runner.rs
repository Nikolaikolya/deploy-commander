/*!
# Подмодуль Runner

Отвечает за запуск цепочек команд и обработку результатов:

- Выполнение цепочек команд
- Обработка ошибок и выполнение команд отката
- Запись результатов в историю деплоев
- Кэширование глобальных переменных для повышения производительности
- Расширенное логирование процесса выполнения команд
*/

use crate::config::Config;
use crate::events::{EventEmitter, EventType};
use crate::executor::chain_builder;
use crate::settings;
use crate::storage;
use anyhow::{Context, Result};
use chrono;
use command_system::CommandResult;
use log::{error, info, trace, warn};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::Instant;

/// Проверяет существование и создает рабочую директорию при необходимости
///
/// # Параметры
///
/// * `path` - Путь к рабочей директории
///
/// # Возвращаемое значение
///
/// Результат создания директории или ошибку
pub fn setup_working_directory(dir: &str) -> Result<()> {
    let dir_path = Path::new(dir);
    if !dir_path.exists() {
        info!("Рабочая директория '{}' не существует, создаем...", dir);
        fs::create_dir_all(dir_path)
            .with_context(|| format!("Не удалось создать рабочую директорию: {}", dir))?;
        info!("Рабочая директория '{}' успешно создана", dir);
    }
    Ok(())
}

/// Определяет файл глобальных переменных на основе явных указаний или настроек
///
/// # Параметры
///
/// * `explicit_path` - Явно указанный путь к файлу переменных или None
///
/// # Возвращаемое значение
///
/// Опциональный путь к файлу глобальных переменных
fn determine_global_variables_file(explicit_path: Option<&str>) -> Option<&str> {
    // Если файл указан явно, используем его
    if let Some(path) = explicit_path {
        if Path::new(path).exists() {
            trace!(
                "Используется явно указанный глобальный файл переменных: {}",
                path
            );
            return Some(path);
        } else {
            warn!("Указанный глобальный файл переменных не найден: {}", path);
            return None;
        }
    }

    // Пытаемся получить путь из настроек
    match settings::get_settings(settings::DEFAULT_SETTINGS_PATH) {
        Ok(settings) => {
            let variables_file = settings.variables_file.clone();
            if Path::new(&variables_file).exists() {
                trace!(
                    "Используется глобальный файл переменных из настроек: {}",
                    variables_file
                );
                let path: &str = Box::leak(variables_file.into_boxed_str());
                Some(path)
            } else {
                warn!(
                    "Глобальный файл переменных из настроек не найден: {}",
                    variables_file
                );
                None
            }
        }
        Err(e) => {
            warn!("Ошибка загрузки настроек для глобальных переменных: {}", e);
            None
        }
    }
}

/// Настраивает рабочую директорию для деплоя
///
/// # Параметры
///
/// * `config` - Конфигурация деплоя
/// * `deployment_name` - Имя деплоя
///
/// # Возвращаемое значение
///
/// Результат настройки рабочей директории
fn setup_deployment_directory(config: &Config, deployment_name: &str) -> Result<()> {
    // Проверяем и настраиваем рабочую директорию
    if let Some(dir) = config
        .find_deployment(deployment_name)
        .and_then(|d| d.working_dir.as_deref())
    {
        match setup_working_directory(dir) {
            Ok(_) => {
                trace!("Рабочая директория '{}' проверена и готова", dir);
                Ok(())
            }
            Err(e) => {
                warn!("Проблема с рабочей директорией '{}': {}", dir, e);
                Err(e)
            }
        }
    } else {
        trace!(
            "Для деплоя '{}' не указана рабочая директория, используется текущая",
            deployment_name
        );
        Ok(())
    }
}

/// Получает путь к файлу истории деплоев из настроек
///
/// # Возвращаемое значение
///
/// Путь к файлу истории деплоев
fn get_history_path() -> String {
    match settings::get_settings(settings::DEFAULT_SETTINGS_PATH) {
        Ok(settings) => settings.history_file,
        Err(_) => {
            warn!("Ошибка загрузки настроек, используется путь к истории деплоев по умолчанию");
            settings::DEFAULT_HISTORY_FILE.to_string()
        }
    }
}

/// Выполняет цепочку команд и обрабатывает результат
///
/// # Параметры
///
/// * `chain` - Цепочка команд для выполнения
/// * `deployment_name` - Имя деплоя
/// * `event_name` - Имя события
/// * `history_path` - Путь к файлу истории деплоев
/// * `start_time` - Время начала выполнения
/// * `emitter` - Эмиттер событий
///
/// # Возвращаемое значение
///
/// Результат выполнения цепочки команд
async fn execute_chain_and_handle_result(
    chain: command_system::chain::CommandChain,
    deployment_name: &str,
    event_name: &str,
    history_path: &str,
    start_time: Instant,
    emitter: EventEmitter,
) -> Result<()> {
    // Выполняем цепочку команд
    let result = chain.execute().await;

    // Проверяем результат выполнения
    match result {
        Ok(chain_result) => {
            // Логируем результаты выполнения каждой команды в цепочке
            for cmd_result in &chain_result.results {
                // Сохраняем детальный вывод в файл лога и выводим в консоль
                save_command_output_to_log(
                    deployment_name,
                    event_name,
                    &cmd_result.command_name,
                    cmd_result,
                );
            }

            // Записываем результат в историю
            if let Err(e) = storage::record_chain_result(
                history_path,
                deployment_name,
                event_name,
                &chain_result,
            ) {
                warn!("Ошибка записи результата в историю: {}", e);
            }

            if chain_result.success {
                // Все команды выполнены успешно
                let duration = start_time.elapsed();
                info!(
                    "Деплой '{}', событие '{}' успешно выполнено за {:.2} секунд",
                    deployment_name,
                    event_name,
                    duration.as_secs_f64()
                );

                emitter.emit(EventType::DeploymentSucceeded {
                    deployment: deployment_name.to_string(),
                    event: event_name.to_string(),
                });

                Ok(())
            } else {
                // Произошла ошибка в одной из команд
                let error_msg = chain_result
                    .error
                    .unwrap_or_else(|| "Неизвестная ошибка".to_string());

                let duration = start_time.elapsed();
                error!(
                    "Деплой '{}', событие '{}' завершилось с ошибками за {:.2} секунд: {}",
                    deployment_name,
                    event_name,
                    duration.as_secs_f64(),
                    error_msg
                );

                emitter.emit(EventType::DeploymentFailed {
                    deployment: deployment_name.to_string(),
                    event: event_name.to_string(),
                });

                Err(anyhow::anyhow!(
                    "Деплой завершился с ошибками: {}",
                    error_msg
                ))
            }
        }
        Err(e) => {
            // Критическая ошибка выполнения цепочки
            let duration = start_time.elapsed();
            error!(
                "Критическая ошибка выполнения деплоя '{}', событие '{}' за {:.2} секунд: {}",
                deployment_name,
                event_name,
                duration.as_secs_f64(),
                e
            );

            emitter.emit(EventType::DeploymentFailed {
                deployment: deployment_name.to_string(),
                event: event_name.to_string(),
            });

            // Записываем ошибку в историю
            if let Err(log_err) = storage::record_deployment(
                history_path,
                deployment_name,
                &format!("error-{}", event_name),
                false,
                Some(e.to_string()),
            ) {
                warn!("Ошибка записи события: {}", log_err);
            }

            Err(anyhow::anyhow!("Критическая ошибка деплоя: {}", e))
        }
    }
}

/// Запускает выполнение цепочки команд для заданного деплоя и события
///
/// # Параметры
///
/// * `config` - Конфигурация деплоя
/// * `deployment_name` - Имя деплоя
/// * `event_name` - Имя события в деплое
/// * `global_variables_file` - Опциональный путь к глобальному файлу переменных
///
/// # Возвращаемое значение
///
/// Возвращает Ok(()) если все команды выполнены успешно, или ошибку если произошла проблема
pub async fn run_commands(
    config: &Config,
    deployment_name: &str,
    event_name: &str,
    global_variables_file: Option<&str>,
) -> Result<()> {
    // Засекаем время начала выполнения для оценки производительности
    let start_time = Instant::now();
    trace!(
        "Начало выполнения деплоя '{}', событие '{}'",
        deployment_name,
        event_name
    );

    // Создаем эмиттер событий
    let emitter = EventEmitter::new();

    // Отправляем событие о начале выполнения
    emitter.emit(EventType::DeploymentStarted {
        deployment: deployment_name.to_string(),
        event: event_name.to_string(),
    });

    // Определяем глобальный файл переменных
    let global_vars_file = determine_global_variables_file(global_variables_file);

    // Настраиваем рабочую директорию
    if let Err(e) = setup_deployment_directory(config, deployment_name) {
        return Err(anyhow::anyhow!(
            "Ошибка настройки рабочей директории: {}",
            e
        ));
    }

    // Создаем цепочку команд
    trace!(
        "Создание цепочки команд для деплоя '{}', событие '{}'",
        deployment_name,
        event_name
    );
    let chain =
        chain_builder::build_command_chain(config, deployment_name, event_name, global_vars_file)?;

    // Выполняем цепочку команд и обрабатываем результат
    info!(
        "Выполнение цепочки команд для деплоя '{}', событие '{}'",
        deployment_name, event_name
    );

    // Получаем путь к истории деплоев
    let history_path = get_history_path();

    // Выполняем цепочку и обрабатываем результат
    execute_chain_and_handle_result(
        chain,
        deployment_name,
        event_name,
        &history_path,
        start_time,
        emitter,
    )
    .await
}

/// Сохраняет детальный вывод команды в файл лога и выводит результат в консоль
///
/// # Параметры
///
/// * `deployment_name` - Имя деплоя
/// * `event_name` - Имя события
/// * `command_name` - Имя команды
/// * `result` - Результат выполнения команды
fn save_command_output_to_log(
    deployment_name: &str,
    event_name: &str,
    command_name: &str,
    result: &CommandResult,
) {
    // Выводим результат выполнения команды в лог
    if result.success {
        let output = result.output.trim();

        // Для больших выводов делаем вывод в несколько строк
        if output.len() > 80 || output.contains('\n') {
            info!("Результат выполнения команды '{}':", command_name);
            for line in output.lines() {
                if !line.is_empty() {
                    info!("│ {}", line);
                }
            }
            if output.lines().count() == 0 {
                info!("│ <пустой вывод>");
            }
            info!("└─ Конец вывода");
        } else {
            info!(
                "Результат выполнения команды '{}': {}",
                command_name, output
            );
        }
    } else {
        let error_msg = result
            .error
            .clone()
            .unwrap_or_else(|| "<неизвестная ошибка>".to_string());

        error!("Ошибка выполнения команды '{}':", command_name);
        error!("├─ Сообщение ошибки: {}", error_msg);

        // Выводим вывод команды, если он есть
        let output = result.output.trim();
        if !output.is_empty() {
            error!("├─ Стандартный вывод команды:");
            for line in output.lines() {
                if !line.is_empty() {
                    error!("│ {}", line);
                }
            }
        } else {
            error!("├─ Стандартный вывод команды: <пустой>");
        }
        error!("└─ Конец вывода");
    }

    // Получаем директорию логов из настроек или используем значение по умолчанию
    let logs_dir = match settings::get_settings(settings::DEFAULT_SETTINGS_PATH) {
        Ok(settings) => settings.logs_dir,
        Err(e) => {
            warn!("Ошибка загрузки настроек для директории логов: {}", e);
            settings::DEFAULT_LOGS_DIR.to_string()
        }
    };

    // Создаем директорию логов, если ее нет
    if !std::path::Path::new(&logs_dir).exists() {
        if let Err(e) = std::fs::create_dir_all(&logs_dir) {
            warn!("Не удалось создать директорию логов {}: {}", logs_dir, e);
            return;
        }
    }

    // Создаем имя файла лога только с датой (один файл на день)
    let current_date = chrono::Local::now().format("%Y%m%d");
    let timestamp = chrono::Local::now().format("%H:%M:%S");
    let filename = format!("{}/{}_commands.log", logs_dir, current_date);

    // Формируем содержимое лога с отметкой времени и информацией о команде
    let log_content = if result.success {
        format!(
            "\n[{}] Деплой: '{}', Событие: '{}', Команда: '{}'\nСтатус: Успех\nВывод:\n{}\n{}\n",
            timestamp, deployment_name, event_name, command_name, 
            result.output.trim(),
            "-".repeat(80)
        )
    } else {
        let error_msg = result
            .error
            .clone()
            .unwrap_or_else(|| "<неизвестная ошибка>".to_string());
        format!(
            "\n[{}] Деплой: '{}', Событие: '{}', Команда: '{}'\nСтатус: Ошибка\nСообщение ошибки:\n{}\nСтандартный вывод:\n{}\n{}\n",
            timestamp, deployment_name, event_name, command_name, 
            error_msg, result.output.trim(),
            "-".repeat(80)
        )
    };

    // Записываем или дописываем лог в файл
    let file_exists = std::path::Path::new(&filename).exists();
    let file_operation_result;
    
    // Если файл существует, дописываем в него
    if file_exists {
        file_operation_result = std::fs::OpenOptions::new()
            .append(true)
            .open(&filename)
            .and_then(|mut file| file.write_all(log_content.as_bytes()));
    } else {
        // Если файла нет, создаем новый
        file_operation_result = std::fs::write(&filename, log_content);
    }

    if let Err(e) = file_operation_result {
        warn!("Не удалось записать лог в файл {}: {}", filename, e);
    } else {
        if file_exists {
            info!(
                "Вывод команды '{}' добавлен в лог: {}",
                command_name, filename
            );
        } else {
            info!(
                "Создан новый лог-файл для команд: {}",
                filename
            );
        }
    }
}
