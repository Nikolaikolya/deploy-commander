use std::fs;
use std::path::Path;
use std::collections::HashMap;
use std::env;
use tokio;
use anyhow::Result;
use command_system::CommandExecution;
use log::info;

// Примечание: Данный пример напрямую не использует internal API проекта deploy-commander,
// а создает свою реализацию для тестирования основного функционала variables_file.

// Загружает переменные из JSON-файла
fn load_variables_from_file(file_path: &str) -> Result<HashMap<String, String>> {
    println!("Загрузка переменных из файла: {}", file_path);
    
    // Читаем содержимое файла
    let content = fs::read_to_string(file_path)?;
    
    // Пробуем распарсить JSON
    let json: serde_json::Value = serde_json::from_str(&content)?;
    
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
    
    println!("Загружено {} переменных из файла: {:?}", vars.len(), vars.keys().collect::<Vec<_>>());
    
    Ok(vars)
}

// Заменяет шаблоны вида {#key} на значения из переменных
fn replace_file_variables(command: &str, variables: &HashMap<String, String>) -> String {
    let mut result = command.to_string();
    
    for (key, value) in variables {
        let placeholder = format!("{{#{}}}", key);
        if result.contains(&placeholder) {
            println!("Заменяем {} на {}", placeholder, value);
            result = result.replace(&placeholder, value);
        }
    }
    
    result
}

#[tokio::main]
async fn main() -> Result<()> {
    // Инициализация команд
    env_logger::init();
    
    // Создаем тестовый файл с переменными, если он не существует
    let test_vars_path = "test_vars.json";
    if !Path::new(test_vars_path).exists() {
        let test_vars = r#"{"TEST_VAR": "test_value", "ANOTHER_VAR": "another_value"}"#;
        fs::write(test_vars_path, test_vars)?;
        println!("Создан тестовый файл {}", test_vars_path);
    }
    
    // Создаем глобальный файл с переменными, если он не существует
    let global_vars_path = "global_test_vars.json";
    if !Path::new(global_vars_path).exists() {
        let global_vars = r#"{"GLOBAL_VAR": "глобальное значение", "COMMON_VAR": "значение из глобального файла"}"#;
        fs::write(global_vars_path, global_vars)?;
        println!("Создан глобальный файл с переменными {}", global_vars_path);
    }
    
    // Устанавливаем переменную окружения для теста
    env::set_var("ENV_TEST_VAR", "значение из переменной окружения");
    
    println!("\n=== ТЕСТИРОВАНИЕ ОБРАБОТКИ VARIABLES_FILE В ПРОЕКТЕ ===\n");
    
    // Тест 1: Загрузка переменных из файла напрямую
    println!("\nТест 1: Загрузка переменных из файла напрямую");
    let vars = load_variables_from_file(test_vars_path)?;
    println!("Успешно загружены переменные из файла:");
    for (key, value) in &vars {
        println!("  {}: {}", key, value);
    }
    
    // Тест 2: Подстановка переменных из файла в команду
    println!("\nТест 2: Подстановка переменных из файла в команду");
    let command_str = "echo 'Переменная из файла: {#TEST_VAR}, другая переменная: {#ANOTHER_VAR}'";
    let replaced_command = replace_file_variables(command_str, &vars);
    println!("Исходная команда: {}", command_str);
    println!("Команда с подстановкой: {}", replaced_command);
    
    // Тест 3: Использование CommandBuilder для выполнения команды
    println!("\nТест 3: Использование CommandBuilder для выполнения команды");
    let cmd_name = format!("var_cmd_{}", chrono::Utc::now().timestamp_millis());
    let command = command_system::CommandBuilder::new(&cmd_name, &replaced_command)
        .execution_mode(command_system::ExecutionMode::Sequential)
        .build();
    
    let result = command.execute().await?;
    println!("Результат выполнения команды с переменными проекта:");
    println!("Успех: {}", result.success);
    println!("Вывод: {}", result.output);
    if let Some(err) = &result.error {
        println!("Ошибка: {}", err);
    }
    
    // Тест 4: Обработка отсутствующих переменных
    println!("\nТест 4: Обработка отсутствующих переменных");
    let command_str = "echo 'Отсутствующая переменная: {#MISSING_VAR}'";
    let replaced_command = replace_file_variables(command_str, &vars);
    println!("Исходная команда: {}", command_str);
    println!("Команда с подстановкой: {}", replaced_command);
    
    // Тест 5: Приоритет локальных переменных над глобальными
    println!("\nТест 5: Приоритет локальных переменных над глобальными");
    let global_vars = load_variables_from_file(global_vars_path)?;
    
    // Создаем локальный файл с дублирующейся переменной
    let local_override_path = "local_override_vars.json";
    fs::write(local_override_path, r#"{"COMMON_VAR": "значение из локального файла"}"#)?;
    
    let local_vars = load_variables_from_file(local_override_path)?;
    
    // Объединяем переменные (локальные имеют приоритет)
    let mut merged_vars = global_vars.clone();
    for (key, value) in &local_vars {
        merged_vars.insert(key.clone(), value.clone());
    }
    
    let command_str = "echo 'Общая переменная: {#COMMON_VAR}'";
    let replaced_command = replace_file_variables(command_str, &merged_vars);
    println!("Исходная команда: {}", command_str);
    println!("Команда с подстановкой: {}", replaced_command);
    
    let cmd_name = format!("var_cmd_{}", chrono::Utc::now().timestamp_millis());
    let command = command_system::CommandBuilder::new(&cmd_name, &replaced_command)
        .execution_mode(command_system::ExecutionMode::Sequential)
        .build();
    
    let result = command.execute().await?;
    println!("Результат выполнения команды с переменными проекта:");
    println!("Успех: {}", result.success);
    println!("Вывод: {}", result.output);
    if let Some(err) = &result.error {
        println!("Ошибка: {}", err);
    }
    
    // Тест 6: Пример из вопроса пользователя (без CommandBuilder.variables_file)
    println!("\nТест 6: Пример из вопроса пользователя (без CommandBuilder.variables_file)");
    let command_str = "echo 'Отсутствует в env: {$MISSING_VAR}, отсутствует в файле: {#MISSING_VAR}'";
    let replaced_command = replace_file_variables(command_str, &vars);
    println!("Исходная команда: {}", command_str);
    println!("Команда с подстановкой переменных из файла: {}", replaced_command);
    
    // Тест 7: Использование CommandBuilder.variables_file напрямую
    println!("\nТест 7: Использование CommandBuilder.variables_file напрямую (если поддерживается)");
    println!("Этот тест запускается только если в CommandBuilder есть метод variables_file");
    
    // Попытка использования метода variables_file, если он доступен
    // Обратите внимание: Этот тест может завершиться ошибкой компиляции, если метод недоступен
    #[cfg(feature = "variables_file_supported")]
    {
        let command = command_system::CommandBuilder::new(
            "missing_test",
            "echo 'Отсутствует в env: {$MISSING_VAR}, отсутствует в файле: {#MISSING_VAR}'"
        )
        .variables_file("test_vars.json")
        .build();
        
        let result = command.execute().await?;
        println!("Результат выполнения: {}", result.output.trim());
    }
    
    // Удаляем временные файлы
    fs::remove_file(local_override_path)?;
    
    Ok(())
} 