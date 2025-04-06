/*!
# Модуль загрузки переменных

Модуль обеспечивает загрузку переменных из различных источников:

- Файлы JSON (`.json`)
- Файлы YAML (`.yml`, `.yaml`) 
- Переменные окружения
- Интерактивные переменные (ввод пользователя)

Модуль предоставляет функции для:
- Обнаружения формата файла
- Загрузки переменных из файлов (JSON, YAML)
- Загрузки переменных из переменных окружения
- Обработки и объединения переменных из разных источников

## Форматы файлов переменных

Поддерживаются два формата файлов:
1. **JSON** - стандартный формат с широкой поддержкой и простым синтаксисом:
   ```json
   {
     "DB_HOST": "localhost",
     "DB_PORT": "5432"
   }
   ```

2. **YAML** - более читаемый формат с поддержкой комментариев и сложных структур:
   ```yaml
   DB_HOST: localhost
   DB_PORT: 5432
   # Это комментарий
   complex_value:
     nested: true
     count: 42
   ```

## Обнаружение формата

Формат файла определяется автоматически по расширению файла:
- `.json` для JSON
- `.yml`, `.yaml` для YAML
- Для других расширений выполняется попытка обработать как JSON, затем как YAML

## Использование

Основная функция для использования в других модулях:
- `load_variables_from_file` - загружает переменные из файла, автоматически определяя формат.
*/

use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use serde_yaml;
use serde_json;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

use super::runner::VariablesFileFormat;

/// Определяет формат файла на основе расширения
///
/// # Параметры
///
/// * `file_path` - Путь к файлу переменных
///
/// # Возвращаемое значение
///
/// Формат файла или ошибка, если формат не поддерживается
fn detect_file_format(file_path: &str) -> Result<VariablesFileFormat> {
    let path = Path::new(file_path);
    
    // Проверяем расширение файла
    if let Some(extension) = path.extension() {
        match extension.to_str().unwrap_or("").to_lowercase().as_str() {
            "json" => {
                debug!("Файл {} определен как JSON по расширению", file_path);
                Ok(VariablesFileFormat::JSON)
            },
            "yaml" | "yml" => {
                debug!("Файл {} определен как YAML по расширению", file_path);
                Ok(VariablesFileFormat::YAML)
            },
            _ => {
                warn!("Неизвестное расширение файла: {:?}, пробуем как JSON", extension);
                Ok(VariablesFileFormat::JSON)
            }
        }
    } else {
        warn!("Файл без расширения: {}, пробуем как JSON", file_path);
        Ok(VariablesFileFormat::JSON)
    }
}

/// Загружает переменные из файла указанного формата
///
/// # Параметры
///
/// * `file_path` - Путь к файлу с переменными
/// * `format` - Формат файла (JSON, YAML или Auto)
///
/// # Возвращаемое значение
///
/// Карта переменных или ошибка
pub fn load_variables_from_file_with_format(
    file_path: &str,
    format: VariablesFileFormat,
) -> Result<HashMap<String, String>> {
    // Проверяем существование файла
    if !Path::new(file_path).exists() {
        return Err(anyhow::anyhow!("Файл переменных не найден: {}", file_path));
    }

    // Читаем содержимое файла
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Ошибка чтения файла {}", file_path))?;

    // Определяем формат, если указан Auto
    let actual_format = if format == VariablesFileFormat::Auto {
        detect_file_format(file_path)?
    } else {
        format
    };

    info!("Загрузка переменных из файла {} в формате {:?}", file_path, actual_format);

    // Парсим содержимое в зависимости от формата
    let variables = match actual_format {
        VariablesFileFormat::JSON => {
            info!("Загружаем переменные из JSON файла: {}", file_path);
            load_from_json(&content, file_path)
        },
        VariablesFileFormat::YAML => {
            info!("Загружаем переменные из YAML файла: {}", file_path);
            load_from_yaml(&content, file_path)
        },
        VariablesFileFormat::Auto => unreachable!(), // Уже обработано выше
    }?;

    // Обрабатываем переменные окружения в значениях
    let mut processed_vars = variables;
    process_env_variables_in_values(&mut processed_vars);

    // Если не вывели ранее, логируем ключи загруженных переменных
    info!(
        "Загружено {} переменных из файла {}: {:?}",
        processed_vars.len(),
        file_path,
        processed_vars.keys().collect::<Vec<_>>()
    );

    Ok(processed_vars)
}

/// Загружает переменные из JSON строки
///
/// # Параметры
///
/// * `content` - JSON строка
/// * `file_path` - Путь к файлу (для логирования)
///
/// # Возвращаемое значение
///
/// Карта переменных или ошибка
fn load_from_json(content: &str, file_path: &str) -> Result<HashMap<String, String>> {
    // Удаляем экранированные кавычки, если они есть
    let cleaned_content = content.replace("\\\"", "\"").replace("\\\\", "\\");

    // Пробуем распарсить JSON
    let json = serde_json::from_str::<serde_json::Value>(&cleaned_content)
        .with_context(|| {
            // Если не удалось распарсить, выводим первые 100 символов для отладки
            let preview = if cleaned_content.len() > 100 {
                format!("{}...", &cleaned_content[..100])
            } else {
                cleaned_content.clone()
            };
            format!(
                "Ошибка парсинга JSON из {}: (начало файла: {})",
                file_path, preview
            )
        })?;

    let mut vars = HashMap::new();

    if let serde_json::Value::Object(map) = json {
        for (key, value) in map {
            if let Some(string_value) = value.as_str() {
                vars.insert(key, string_value.to_string());
            } else {
                vars.insert(key, value.to_string());
            }
        }
    }

    Ok(vars)
}

/// Загружает переменные из YAML строки
///
/// # Параметры
///
/// * `content` - YAML строка
/// * `file_path` - Путь к файлу (для логирования)
///
/// # Возвращаемое значение
///
/// Карта переменных или ошибка
fn load_from_yaml(content: &str, file_path: &str) -> Result<HashMap<String, String>> {
    // Пробуем распарсить YAML
    let yaml = serde_yaml::from_str::<serde_yaml::Value>(content)
        .with_context(|| {
            // Если не удалось распарсить, выводим первые 100 символов для отладки
            let preview = if content.len() > 100 {
                format!("{}...", &content[..100])
            } else {
                content.to_string()
            };
            format!(
                "Ошибка парсинга YAML из {}: (начало файла: {})",
                file_path, preview
            )
        })?;

    let mut vars = HashMap::new();

    if let serde_yaml::Value::Mapping(map) = yaml {
        for (key, value) in map {
            if let Some(key_str) = key.as_str() {
                if let Some(value_str) = value.as_str() {
                    vars.insert(key_str.to_string(), value_str.to_string());
                } else {
                    // Для сложных типов преобразуем в строку через JSON
                    let value_str = match serde_json::to_string(&value) {
                        Ok(s) => s,
                        Err(_) => format!("{:?}", value), // Используем Debug формат если JSON не работает
                    };
                    vars.insert(key_str.to_string(), value_str);
                }
            }
        }
    }

    Ok(vars)
}

/// Заменяет переменные окружения в значениях из файла переменных
///
/// # Параметры
///
/// * `variables` - Карта переменных для обработки
pub fn process_env_variables_in_values(variables: &mut HashMap<String, String>) {
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

/// Объединяет несколько карт переменных с приоритетом более поздних
///
/// # Параметры
///
/// * `maps` - Вектор карт переменных в порядке возрастания приоритета
///
/// # Возвращаемое значение
///
/// Объединенная карта переменных
pub fn merge_variable_maps(maps: Vec<HashMap<String, String>>) -> HashMap<String, String> {
    let mut result = HashMap::new();
    
    for map in maps {
        for (key, value) in map {
            result.insert(key, value);
        }
    }
    
    result
}

/// Загружает переменные из нескольких файлов с приоритетом
///
/// # Параметры
///
/// * `file_paths` - Вектор путей к файлам в порядке возрастания приоритета
///
/// # Возвращаемое значение
///
/// Объединенная карта переменных или ошибка
pub fn load_variables_from_multiple_files(
    file_paths: &[&str],
) -> Result<HashMap<String, String>> {
    let mut all_vars = HashMap::new();
    
    for &file_path in file_paths {
        if Path::new(file_path).exists() {
            match load_variables_from_file_with_format(file_path, VariablesFileFormat::Auto) {
                Ok(vars) => {
                    info!("Загружено {} переменных из файла {}", vars.len(), file_path);
                    // Добавляем переменные с перезаписью существующих
                    for (key, value) in vars {
                        all_vars.insert(key, value);
                    }
                }
                Err(e) => {
                    warn!("Ошибка загрузки переменных из файла {}: {}", file_path, e);
                }
            }
        } else {
            debug!("Файл переменных не найден: {}", file_path);
        }
    }
    
    Ok(all_vars)
} 