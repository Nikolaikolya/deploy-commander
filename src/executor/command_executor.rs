/*!
# Подмодуль Command Executor

Отвечает за настройку и выполнение отдельных команд:

- Создание команд с заданными параметрами
- Настройка рабочих директорий и переменных окружения
- Добавление команд отката
- Поддержка переменных для подстановки значений
*/

use command_system::{CommandBuilder, CommandExecution, ExecutionMode};
use log::info;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Создает команду с заданными параметрами
///
/// # Параметры
///
/// * `name` - Уникальное имя команды
/// * `command` - Строка с командой для выполнения
/// * `working_dir` - Опциональная рабочая директория
/// * `env_vars` - Переменные окружения для команды
/// * `rollback_command` - Опциональная команда отката при ошибке
/// * `interactive` - Флаг интерактивного режима
/// * `inputs` - Предопределенные ответы на интерактивные запросы
/// * `variables_file` - Путь к файлу с переменными для подстановки
/// * `global_variables_file` - Опциональный путь к глобальному файлу с переменными
///
/// # Возвращаемое значение
///
/// Возвращает настроенную команду, готовую к выполнению
pub fn create_command(
    name: &str,
    command: &str,
    working_dir: Option<&str>,
    env_vars: &[(String, String)],
    rollback_command: Option<&str>,
    interactive: bool,
    inputs: Option<HashMap<String, String>>,
    variables_file: Option<&str>,
    global_variables_file: Option<&str>,
) -> command_system::command::ShellCommand {
    let mut builder = CommandBuilder::new(name, command).execution_mode(ExecutionMode::Sequential);

    // Добавляем рабочую директорию, если указана
    if let Some(dir) = working_dir {
        builder = builder.working_dir(dir);
    }

    // Добавляем переменные окружения
    for (key, value) in env_vars {
        builder = builder.env_var(key, value);
    }

    // Добавляем команду отката, если указана
    if let Some(rollback) = rollback_command {
        builder = builder.rollback(rollback);
    }

    // Загружаем переменные из файла, если указан
    if let Some(file_path) = variables_file {
        if Path::new(file_path).exists() {
            // Чтение переменных из JSON-файла
            match load_variables_from_file(file_path, global_variables_file) {
                Ok(vars) => {
                    info!(
                        "Загружено {} переменных для команды: {}",
                        vars.len(),
                        command
                    );
                    for (key, value) in vars {
                        builder = builder.env_var(&key, &value);
                    }
                }
                Err(e) => {
                    info!("Ошибка загрузки переменных для команды: {}", e);
                }
            }
        } else {
            info!("Файл переменных не найден: {}", file_path);
        }
    } else if let Some(global_path) = global_variables_file {
        // Если локальный файл не указан, но указан глобальный
        if Path::new(global_path).exists() {
            match load_variables_from_single_file(global_path) {
                Ok(vars) => {
                    info!(
                        "Загружено {} глобальных переменных для команды: {}",
                        vars.len(),
                        command
                    );
                    for (key, value) in vars {
                        builder = builder.env_var(&key, &value);
                    }
                }
                Err(e) => {
                    info!("Ошибка загрузки глобальных переменных: {}", e);
                }
            }
        } else {
            info!("Глобальный файл переменных не найден: {}", global_path);
        }
    }

    // Если команда интерактивная и есть предопределенные ответы,
    // заменяем шаблоны в команде на значения из inputs
    if interactive && inputs.is_some() {
        let inputs_map = inputs.unwrap();
        for (key, value) in inputs_map {
            // Заменяем шаблоны вида {key} на их значения
            let placeholder = format!("{{{}}}", key);
            if command.contains(&placeholder) {
                builder = builder.env_var(&key, &value);
            }
        }
    }

    builder.build()
}

/// Загружает переменные из JSON-файла
///
/// # Параметры
///
/// * `file_path` - Путь к файлу с переменными
/// * `global_variables_path` - Опциональный путь к глобальному файлу с переменными
///
/// # Возвращаемое значение
///
/// Хэш-карта с переменными или ошибка
fn load_variables_from_file(
    file_path: &str,
    global_variables_path: Option<&str>,
) -> anyhow::Result<HashMap<String, String>> {
    let mut vars = HashMap::new();

    // Сначала загружаем глобальные переменные, если указаны
    if let Some(global_path) = global_variables_path {
        if Path::new(global_path).exists() {
            match load_variables_from_single_file(global_path) {
                Ok(global_vars) => {
                    info!(
                        "Загружено {} глобальных переменных из файла: {}",
                        global_vars.len(),
                        global_path
                    );
                    vars.extend(global_vars);
                }
                Err(e) => {
                    info!(
                        "Ошибка загрузки глобальных переменных из файла {}: {}",
                        global_path, e
                    );
                }
            }
        } else {
            info!("Глобальный файл переменных не найден: {}", global_path);
        }
    }

    // Затем загружаем локальные переменные, которые могут переопределить глобальные
    if Path::new(file_path).exists() {
        match load_variables_from_single_file(file_path) {
            Ok(local_vars) => {
                info!(
                    "Загружено {} локальных переменных из файла: {}",
                    local_vars.len(),
                    file_path
                );
                vars.extend(local_vars); // Переопределяем глобальные переменные локальными
            }
            Err(e) => {
                info!(
                    "Ошибка загрузки локальных переменных из файла {}: {}",
                    file_path, e
                );
            }
        }
    } else {
        info!("Локальный файл переменных не найден: {}", file_path);
    }

    Ok(vars)
}

/// Загружает переменные из одного JSON-файла
///
/// # Параметры
///
/// * `file_path` - Путь к файлу с переменными
///
/// # Возвращаемое значение
///
/// Хэш-карта с переменными или ошибка
fn load_variables_from_single_file(file_path: &str) -> anyhow::Result<HashMap<String, String>> {
    // Читаем содержимое файла
    let content = fs::read_to_string(file_path).map_err(|e| {
        info!("Ошибка чтения файла {}: {}", file_path, e);
        e
    })?;

    // Удаляем экранированные кавычки, если они есть
    let cleaned_content = content.replace("\\\"", "\"").replace("\\\\", "\\");

    // Пробуем распарсить JSON
    let json = serde_json::from_str::<Value>(&cleaned_content).map_err(|e| {
        // Если не удалось распарсить, выводим первые 100 символов для отладки
        let preview = if cleaned_content.len() > 100 {
            format!("{}...", &cleaned_content[..100])
        } else {
            cleaned_content.clone()
        };
        info!(
            "Ошибка парсинга JSON из {}: {} (начало файла: {})",
            file_path, e, preview
        );
        e
    })?;

    let mut vars = HashMap::new();

    if let Value::Object(map) = json {
        for (key, value) in map {
            if let Some(string_value) = value.as_str() {
                vars.insert(key, string_value.to_string());
            } else {
                vars.insert(key, value.to_string());
            }
        }
    }

    info!(
        "Успешно загружены переменные из файла {}: {:?}",
        file_path,
        vars.keys().collect::<Vec<_>>()
    );

    Ok(vars)
}

/// Простая версия создания команды для обратной совместимости
pub fn create_simple_command(
    name: &str,
    command: &str,
    working_dir: Option<&str>,
    env_vars: &[(String, String)],
    rollback_command: Option<&str>,
) -> command_system::command::ShellCommand {
    create_command(
        name,
        command,
        working_dir,
        env_vars,
        rollback_command,
        false,
        None,
        None,
        None,
    )
}

/// Выполняет простую команду и возвращает результат
///
/// # Параметры
///
/// * `command` - Строка с командой для выполнения
///
/// # Возвращаемое значение
///
/// Результат выполнения команды или ошибку
#[allow(dead_code)]
pub async fn execute_simple_command(command: &str) -> anyhow::Result<String> {
    let cmd_name = format!("simple_cmd_{}", chrono::Utc::now().timestamp_millis());
    let command = create_simple_command(&cmd_name, command, None, &[], None);

    match command.execute().await {
        Ok(result) => {
            if result.success {
                Ok(result.output)
            } else {
                Err(anyhow::anyhow!(
                    "Команда завершилась с ошибкой: {}",
                    result
                        .error
                        .unwrap_or_else(|| "<неизвестная ошибка>".to_string())
                ))
            }
        }
        Err(e) => Err(anyhow::anyhow!("Ошибка выполнения команды: {}", e)),
    }
}

/// Выполняет команду с переменными
///
/// # Параметры
///
/// * `command` - Строка с командой для выполнения, содержащая переменные
/// * `variables` - Карта переменных вида "имя" -> "значение"
/// * `variables_file` - Опциональный путь к файлу с переменными
///
/// # Возвращаемое значение
///
/// Результат выполнения команды или ошибку
#[allow(dead_code)]
pub async fn execute_command_with_variables(
    command: &str,
    variables: HashMap<String, String>,
    variables_file: Option<&str>,
) -> anyhow::Result<String> {
    let cmd_name = format!("var_cmd_{}", chrono::Utc::now().timestamp_millis());

    // Настраиваем строитель команды
    let mut builder =
        CommandBuilder::new(&cmd_name, command).execution_mode(ExecutionMode::Sequential);

    // Добавляем переменные окружения
    for (key, value) in &variables {
        builder = builder.env_var(key, value);
    }

    // Загружаем переменные из файла, если указан
    if let Some(file_path) = variables_file {
        if Path::new(file_path).exists() {
            // Загрузка переменных из JSON-файла
            match load_variables_from_file(file_path, None) {
                Ok(vars) => {
                    info!(
                        "Загружено {} переменных из файла: {}",
                        vars.len(),
                        file_path
                    );
                    for (key, value) in vars {
                        builder = builder.env_var(&key, &value);
                    }
                }
                Err(e) => {
                    info!("Ошибка загрузки переменных из файла {}: {}", file_path, e);
                }
            }
        } else {
            info!("Файл переменных не найден: {}", file_path);
        }
    }

    // Строим и выполняем команду
    let command = builder.build();

    match command.execute().await {
        Ok(result) => {
            if result.success {
                Ok(result.output)
            } else {
                Err(anyhow::anyhow!(
                    "Команда завершилась с ошибкой: {}",
                    result
                        .error
                        .unwrap_or_else(|| "<неизвестная ошибка>".to_string())
                ))
            }
        }
        Err(e) => Err(anyhow::anyhow!("Ошибка выполнения команды: {}", e)),
    }
}
