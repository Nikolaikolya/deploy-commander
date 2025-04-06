/*!
# Подмодуль Command Executor

Отвечает за настройку и выполнение отдельных команд:

- Создание команд с заданными параметрами
- Настройка рабочих директорий и переменных окружения
- Добавление команд отката
- Поддержка переменных для подстановки значений
*/

use command_system::{CommandBuilder, CommandExecution, ExecutionMode};
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::env;
use std::path::Path;
use tokio::io::AsyncReadExt;

use super::runner::VariablesFileFormat;
use super::variable_loader;

/// Получает форматированное сообщение об ошибке из опционального поля error
fn format_error_message(error: Option<String>) -> String {
    error.unwrap_or_else(|| "<неизвестная ошибка>".to_string())
}

/// Заменяет шаблоны вида {#key} на значения из переменных
fn replace_file_variables(command: &str, variables: &HashMap<String, String>) -> String {
    // Проверяем, содержит ли команда шаблоны
    if !command.contains("{#") {
        return command.to_string();
    }
    
    let mut result = command.to_string();
    
    // Логируем все доступные переменные для отладки
    let keys = variables.keys().collect::<Vec<_>>();
    debug!("Доступные переменные для подстановки: {:?}", keys);
    
    // Собираем все замены для однопроходной обработки
    let mut replacements = Vec::new();
    
    for (key, value) in variables {
        let placeholder = format!("{{#{}}}", key);
        if result.contains(&placeholder) {
            debug!("Найден плейсхолдер {} для замены на {}", placeholder, value);
            replacements.push((placeholder, value.clone()));
        }
    }
    
    // Если есть замены, логируем их и выполняем
    if !replacements.is_empty() {
        info!("Выполняется {} замен в команде: {}", replacements.len(), command);
        
        for (placeholder, value) in replacements {
            debug!("Заменяем {} на {}", placeholder, value);
            result = result.replace(&placeholder, &value);
        }
    }
    
    // Логируем неразрешенные плейсхолдеры
    if result.contains("{#") {
        let mut unresolved = Vec::new();
        let mut start = 0;
        
        while let Some(pos) = result[start..].find("{#") {
            let real_pos = start + pos;
            if let Some(end) = result[real_pos..].find("}") {
                let placeholder = &result[real_pos..(real_pos + end + 1)];
                unresolved.push(placeholder);
                start = real_pos + end + 1;
            } else {
                break;
            }
        }
        
        if !unresolved.is_empty() {
            warn!("Неразрешенные плейсхолдеры в команде: {:?}", unresolved);
        }
    }
    
    debug!("Результат подстановки: {}", result);
    result
}

/// Заменяет переменные окружения в значениях из файла переменных
fn process_env_variables_in_values(variables: &mut HashMap<String, String>) {
    for value in variables.values_mut() {
        if value.contains("${") && value.contains("}") {
            let mut processed = value.clone();
            let mut start_idx = 0;
            
            while let Some(start) = processed[start_idx..].find("${") {
                let real_start = start_idx + start;
                if let Some(end) = processed[real_start..].find("}") {
                    let real_end = real_start + end + 1;
                    let env_var_expr = &processed[real_start..real_end];
                    let env_var_name = &env_var_expr[2..env_var_expr.len()-1];
                    
                    if let Ok(env_value) = env::var(env_var_name) {
                        debug!("Заменяем переменную окружения {} на {} в значении", env_var_expr, env_value);
                        processed = processed.replace(env_var_expr, &env_value);
                    } else {
                        debug!("Переменная окружения {} не найдена, оставляем без изменений", env_var_name);
                        start_idx = real_end;
                    }
                } else {
                    break;
                }
            }
            
            if &processed != value {
                debug!("Значение изменено с учетом переменных окружения: {} -> {}", value, processed);
                *value = processed;
            }
        }
    }
}

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
    // Загружаем переменные из файлов, если указаны
    let mut variables = HashMap::new();
    let mut command_str = command.to_string();

    // Загружаем переменные из файлов
    let mut file_paths = Vec::new();
    
    // Сначала глобальные, потом локальные для приоритета
    if let Some(global_path) = global_variables_file {
        file_paths.push(global_path);
    }
    if let Some(local_path) = variables_file {
        file_paths.push(local_path);
    }
    
    if !file_paths.is_empty() {
        match variable_loader::load_variables_from_multiple_files(&file_paths) {
            Ok(vars) => {
                info!(
                    "Загружено {} переменных для команды из {} файлов",
                    vars.len(),
                    file_paths.len()
                );
                variables = vars;
            }
            Err(e) => {
                warn!("Ошибка загрузки переменных: {}", e);
            }
        }
    }

    // Заменяем переменные в формате {#key} на их значения
    if !variables.is_empty() && command.contains("{#") {
        command_str = replace_file_variables(command, &variables);
        debug!("Команда с подстановкой: {}", command_str);
    }

    // Создаем builder
    let mut builder = CommandBuilder::new(name, &command_str).execution_mode(ExecutionMode::Sequential);

    // Добавляем рабочую директорию, если указана
    if let Some(dir) = working_dir {
        builder = builder.working_dir(dir);
    }

    // Добавляем переменные окружения
    for (key, value) in env_vars {
        builder = builder.env_var(key, value);
    }

    // Добавляем переменные из файлов в env_vars для стандартных шаблонов {key}
    for (key, value) in &variables {
        builder = builder.env_var(key, value);
    }

    // Добавляем команду отката, если указана
    if let Some(rollback) = rollback_command {
        builder = builder.rollback(rollback);
    }

    // Если команда интерактивная и есть предопределенные ответы,
    // заменяем шаблоны вида {key} на их значения
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
/// Устаревшая функция, используйте variable_loader::load_variables_from_file_with_format
///
/// # Параметры
///
/// * `file_path` - Путь к файлу с переменными
/// * `global_variables_path` - Опциональный путь к глобальному файлу с переменными
///
/// # Возвращаемое значение
///
/// Хэш-карта с переменными или ошибка
#[deprecated(since = "0.2.0", note = "Используйте variable_loader::load_variables_from_file_with_format")]
pub fn load_variables_from_file(
    file_path: &str,
    global_variables_path: Option<&str>,
) -> anyhow::Result<HashMap<String, String>> {
    // Собираем пути к файлам для загрузки
    let mut file_paths = Vec::new();
    
    // Сначала глобальные, потом локальные для приоритета
    if let Some(global_path) = global_variables_path {
        file_paths.push(global_path);
    }
    file_paths.push(file_path);
    
    variable_loader::load_variables_from_multiple_files(&file_paths)
}

/// Загружает переменные из одного JSON-файла
///
/// Устаревшая функция, используйте variable_loader::load_variables_from_file_with_format
///
/// # Параметры
///
/// * `file_path` - Путь к файлу с переменными
///
/// # Возвращаемое значение
///
/// Хэш-карта с переменными или ошибка
#[deprecated(since = "0.2.0", note = "Используйте variable_loader::load_variables_from_file_with_format")]
pub fn load_variables_from_single_file(file_path: &str) -> anyhow::Result<HashMap<String, String>> {
    variable_loader::load_variables_from_file_with_format(
        file_path, 
        VariablesFileFormat::JSON
    )
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
    info!("Выполнение простой команды: {}", command);
    let command = create_simple_command(&cmd_name, command, None, &[], None);

    match command.execute().await {
        Ok(result) => {
            if result.success {
                info!("Команда '{}' успешно выполнена. Вывод: {}", cmd_name, result.output.trim());
                Ok(result.output)
            } else {
                let err_msg = format_error_message(result.error);
                error!("Команда '{}' завершилась с ошибкой: {}", cmd_name, err_msg);
                Err(anyhow::anyhow!("Команда завершилась с ошибкой: {}", err_msg))
            }
        }
        Err(e) => {
            error!("Ошибка выполнения команды '{}': {}", cmd_name, e);
            Err(anyhow::anyhow!("Ошибка выполнения команды: {}", e))
        }
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
    info!("Выполнение команды с переменными: {}", command);

    // Настраиваем строитель команды
    let mut builder =
        CommandBuilder::new(&cmd_name, command).execution_mode(ExecutionMode::Sequential);

    // Добавляем переменные окружения
    for (key, value) in &variables {
        builder = builder.env_var(key, value);
        debug!("Добавлена переменная окружения: {}={}", key, value);
    }

    // Загружаем переменные из файла
    if let Some(file_path) = variables_file {
        match variable_loader::load_variables_from_file_with_format(file_path, VariablesFileFormat::Auto) {
            Ok(vars) => {
                info!("Загружено {} переменных из файла {}", vars.len(), file_path);
                for (key, value) in vars {
                    builder = builder.env_var(&key, &value);
                    debug!("Добавлена переменная из файла: {}={}", key, value);
                }
            }
            Err(e) => {
                warn!("Ошибка загрузки переменных из файла {}: {}", file_path, e);
            }
        }
    }

    // Строим и выполняем команду
    let command = builder.build();

    match command.execute().await {
        Ok(result) => {
            if result.success {
                info!("Команда с переменными '{}' успешно выполнена. Вывод: {}", cmd_name, result.output.trim());
                Ok(result.output)
            } else {
                let err_msg = format_error_message(result.error);
                error!("Команда с переменными '{}' завершилась с ошибкой: {}", cmd_name, err_msg);
                Err(anyhow::anyhow!("Команда завершилась с ошибкой: {}", err_msg))
            }
        }
        Err(e) => {
            error!("Ошибка выполнения команды с переменными '{}': {}", cmd_name, e);
            Err(anyhow::anyhow!("Ошибка выполнения команды: {}", e))
        }
    }
}

/// Выполняет команду с указанными параметрами
///
/// # Параметры
///
/// * `name` - Имя команды
/// * `command` - Команда для выполнения
/// * `working_dir` - Опциональная рабочая директория
/// * `description` - Опциональное описание команды
/// * `environment` - Опциональные переменные окружения
/// * `variables` - Опциональные переменные для подстановки
/// * `mode` - Режим выполнения команды
/// * `prompt_on_fail` - Запрашивать подтверждение при ошибке
///
/// # Возвращаемое значение
///
/// Результат выполнения команды или ошибка
pub async fn execute_command(
    name: &str,
    command: &str,
    working_dir: Option<&str>,
    description: Option<&str>,
    environment: Option<&[String]>,
    variables: Option<&HashMap<String, String>>,
    mode: ExecutionMode,
    prompt_on_fail: bool,
) -> anyhow::Result<command_system::CommandResult> {
    debug!(
        "Выполнение команды '{}' с описанием '{:?}'",
        command, description
    );

    // Создаем и настраиваем команду
    let mut builder = CommandBuilder::new(name, command);

    if let Some(dir) = working_dir {
        debug!("Установка рабочей директории: {}", dir);
        builder.working_dir(dir);
    }

    if let Some(desc) = description {
        debug!("Установка описания: {}", desc);
        builder.description(desc);
    }

    if let Some(env_vars) = environment {
        debug!("Установка переменных окружения: {:?}", env_vars);
        for env_var in env_vars {
            if let Some(pos) = env_var.find('=') {
                let (key, value) = env_var.split_at(pos);
                // Пропускаем символ '='
                let value = &value[1..];
                builder.env(key, value);
            } else {
                warn!("Некорректная переменная окружения: {}", env_var);
            }
        }
    }

    if let Some(vars) = variables {
        debug!("Установка переменных для подстановки: {:?}", vars);
        for (key, value) in vars {
            builder.variable(key.clone(), value.clone());
        }
    }

    // Устанавливаем режим выполнения
    builder.mode(mode);
    builder.prompt_on_fail(prompt_on_fail);

    let command = builder.build();
    debug!("Команда готова к выполнению");

    // Выполняем команду
    match command.execute().await {
        Ok(result) => {
            debug!(
                "Команда выполнена с результатом: success={}, output={}",
                result.success, result.output
            );
            Ok(result)
        }
        Err(e) => {
            error!("Ошибка выполнения команды: {}", e);
            Err(anyhow::anyhow!("Ошибка выполнения: {}", e))
        }
    }
}

/// Расширяет CommandBuilder переменными из файла
///
/// # Параметры
///
/// * `builder` - Строитель команды
/// * `file_path` - Путь к файлу переменных
/// * `format` - Формат файла переменных
///
/// # Возвращаемое значение
///
/// Обновленный строитель команды или ошибка
pub async fn extend_builder_with_variables_file(
    builder: &mut CommandBuilder,
    file_path: &str,
    format: VariablesFileFormat,
) -> anyhow::Result<()> {
    info!("1111111111111111 Загружаем переменные из файла: {}", file_path);
    
    // Файл для загрузки переменных
    let mut file = match tokio::fs::File::open(file_path).await {
        Ok(file) => {
            info!("3333333333333333 Открыт файл: {:?}", file);
            file
        }
        Err(e) => {
            warn!("Ошибка открытия файла переменных '{}': {}", file_path, e);
            return Err(anyhow::anyhow!("Ошибка открытия файла: {}", e));
        }
    };
    
    info!("4444444444444444 Загружаем файл: {:?}", file);
    
    let mut content = String::new();
    match file.read_to_string(&mut content).await {
        Ok(_) => {
            info!("5555555555555555 Прочитан файл: {:?}", content);
        }
        Err(e) => {
            warn!("Ошибка чтения файла переменных '{}': {}", file_path, e);
            return Err(anyhow::anyhow!("Ошибка чтения файла: {}", e));
        }
    }
    
    // Определяем формат файла
    let actual_format = if format == VariablesFileFormat::Auto {
        // Автоопределение формата по расширению
        let path = Path::new(file_path);
        match path.extension().and_then(|e| e.to_str()) {
            Some("json") => VariablesFileFormat::JSON,
            Some("yml") | Some("yaml") => VariablesFileFormat::YAML,
            _ => {
                info!("Неизвестное расширение файла, пробуем как JSON");
                VariablesFileFormat::JSON 
            }
        }
    } else {
        format
    };
    
    info!("Используем формат файла: {:?}", actual_format);
    
    // Загружаем переменные в зависимости от формата
    let variables = match actual_format {
        VariablesFileFormat::JSON => {
            // Пробуем загрузить как JSON
            info!("6666666666666666 Разбираем JSON...");
            match serde_json::from_str::<HashMap<String, String>>(&content) {
                Ok(vars) => vars,
                Err(e) => {
                    warn!("Ошибка парсинга JSON из '{}': {}", file_path, e);
                    return Err(anyhow::anyhow!("Ошибка парсинга JSON: {}", e));
                }
            }
        }
        VariablesFileFormat::YAML => {
            // Пробуем загрузить как YAML
            info!("6666666666666666 Разбираем YAML...");
            // Сначала разбираем в общую структуру YAML
            let yaml_value: serde_yaml::Value = match serde_yaml::from_str(&content) {
                Ok(value) => value,
                Err(e) => {
                    warn!("Ошибка парсинга YAML из '{}': {}", file_path, e);
                    return Err(anyhow::anyhow!("Ошибка парсинга YAML: {}", e));
                }
            };
            
            info!("Разобран YAML: {:?}", yaml_value);
            
            // Затем преобразуем в карту строк
            let mut vars = HashMap::new();
            if let serde_yaml::Value::Mapping(map) = yaml_value {
                for (key, value) in map {
                    if let Some(key_str) = key.as_str() {
                        let value_str = match value {
                            serde_yaml::Value::String(s) => s,
                            serde_yaml::Value::Number(n) => n.to_string(),
                            serde_yaml::Value::Bool(b) => b.to_string(),
                            serde_yaml::Value::Sequence(_) => serde_json::to_string(&value)
                                .unwrap_or_else(|_| format!("{:?}", value)),
                            serde_yaml::Value::Mapping(_) => serde_json::to_string(&value)
                                .unwrap_or_else(|_| format!("{:?}", value)),
                            _ => format!("{:?}", value),
                        };
                        vars.insert(key_str.to_string(), value_str);
                    }
                }
            } else {
                warn!("YAML не содержит карту значений верхнего уровня");
                return Err(anyhow::anyhow!("YAML не содержит карту значений верхнего уровня"));
            }
            vars
        }
        _ => unreachable!(),
    };
    
    info!("2222222222222222 Загружены переменные из файла: {:?}", variables);
    
    // Добавляем переменные в builder
    for (key, value) in variables {
        builder.variable(key, value);
    }
    
    Ok(())
}

/// Загружает переменные из файла JSON или YAML
///
/// # Параметры
///
/// * `file_path` - Путь к файлу переменных
/// * `format` - Формат файла переменных (JSON, YAML, Auto)
///
/// # Возвращаемое значение
///
/// Карта переменных или ошибка
pub fn load_variables_from_file(
    file_path: &str,
    format: VariablesFileFormat,
) -> anyhow::Result<HashMap<String, String>> {
    variable_loader::load_variables_from_file_with_format(file_path, format)
}

/// Загружает переменные из файла, автоматически определяя формат
///
/// # Параметры
///
/// * `file_path` - Путь к файлу переменных
///
/// # Возвращаемое значение
///
/// Карта переменных или ошибка
pub fn load_variables_from_single_file(file_path: &str) -> anyhow::Result<HashMap<String, String>> {
    variable_loader::load_variables_from_file_with_format(file_path, VariablesFileFormat::Auto)
}
