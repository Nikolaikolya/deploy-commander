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
use log::{error, info, trace, warn};
use std::fs;
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
            // Записываем результат в историю
            if let Err(e) = storage::record_chain_result(
                history_path,
                deployment_name,
                event_name,
                &chain_result,
            ) {
                warn!("Ошибка записи результата: {}", e);
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

                // Логируем результаты всех команд в цепочке для отображения вывода
                for (index, cmd_result) in chain_result.results.iter().enumerate() {
                    info!(
                        "Результат команды #{}: {}",
                        index + 1,
                        cmd_result.output.trim()
                    );
                }

                Ok(())
            } else {
                // Произошла ошибка в одной из команд
                let error_message = match &chain_result.error {
                    Some(msg) => msg.clone(),
                    None => {
                        // Находим первую команду, которая завершилась с ошибкой
                        let failed_cmd = chain_result
                            .results
                            .iter()
                            .find(|r| !r.success)
                            .and_then(|r| r.error.as_ref())
                            .unwrap_or(&"<неизвестная ошибка>".to_string())
                            .clone();
                        format!("Ошибка выполнения команды: {}", failed_cmd)
                    }
                };

                error!(
                    "Деплой '{}', событие '{}' завершилось с ошибкой: {}",
                    deployment_name, event_name, error_message
                );

                emitter.emit(EventType::DeploymentFailed {
                    deployment: deployment_name.to_string(),
                    event: event_name.to_string(),
                    error: error_message.clone(),
                });

                // Логируем результаты команд, включая информацию об ошибке
                for (index, cmd_result) in chain_result.results.iter().enumerate() {
                    if cmd_result.success {
                        info!(
                            "Команда #{}: {}",
                            index + 1,
                            cmd_result.output.trim()
                        );
                    } else {
                        error!(
                            "Ошибка в команде #{}: {}",
                            index + 1,
                            cmd_result.error.as_ref().unwrap_or(&"<неизвестная ошибка>".to_string())
                        );
                    }
                }

                Err(anyhow::anyhow!(error_message))
            }
        }
        Err(e) => {
            error!(
                "Ошибка при выполнении цепочки команд для деплоя '{}': {}",
                deployment_name, e
            );

            emitter.emit(EventType::DeploymentFailed {
                deployment: deployment_name.to_string(),
                event: event_name.to_string(),
                error: e.to_string(),
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

            Err(anyhow::anyhow!("Ошибка выполнения команд: {}", e))
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
    info!(
        "Запуск команд деплоя '{}', событие '{}'",
        deployment_name, event_name
    );

    // Строим цепочку команд
    let chain = chain_builder::build_command_chain(
        config,
        deployment_name,
        event_name,
        global_variables_file,
    )?;

    // Запускаем цепочку команд
    info!("Начало выполнения цепочки команд...");
    
    let result = chain.execute().await;

    match result {
        Ok(chain_result) => {
            if chain_result.success {
                info!("Успешно выполнены все команды цепочки");
                
                // Логируем результаты всех команд в цепочке
                for (index, cmd_result) in chain_result.results.iter().enumerate() {
                    info!(
                        "Результат команды #{}: успех={}, вывод={}",
                        index + 1,
                        cmd_result.success,
                        cmd_result.output.trim()
                    );
                }
                
                Ok(())
            } else {
                let error_message = match &chain_result.error {
                    Some(msg) => msg.clone(),
                    None => {
                        let failed_cmd = chain_result
                            .results
                            .iter()
                            .find(|r| !r.success)
                            .and_then(|r| r.error.as_ref())
                            .unwrap_or(&"<неизвестная ошибка>".to_string())
                            .clone();
                        format!("Ошибка выполнения команды: {}", failed_cmd)
                    }
                };
                
                error!("Выполнение цепочки команд завершилось с ошибкой: {}", error_message);
                
                // Логируем результаты всех выполненных команд в цепочке
                for (index, cmd_result) in chain_result.results.iter().enumerate() {
                    if cmd_result.success {
                        info!(
                            "Результат команды #{}: успех=true, вывод={}",
                            index + 1,
                            cmd_result.output.trim()
                        );
                    } else {
                        error!(
                            "Ошибка в команде #{}: {}",
                            index + 1,
                            cmd_result.error.as_ref().unwrap_or(&"<неизвестная ошибка>".to_string())
                        );
                    }
                }
                
                Err(anyhow::anyhow!(error_message))
            }
        }
        Err(e) => {
            error!("Ошибка при выполнении цепочки команд: {}", e);
            Err(anyhow::anyhow!("Ошибка выполнения команд: {}", e))
        }
    }
}
