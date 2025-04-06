use std::fs;
use std::path::Path;
use std::collections::HashMap;
use std::env;
use command_system::{CommandBuilder, CommandExecution, ExecutionMode};

// Функция загрузки переменных из файла - упрощенная версия из deploy-commander
fn load_variables_from_file(file_path: &str) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
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
    
    println!("Загружено {} переменных из файла: {}", vars.len(), file_path);
    
    Ok(vars)
}

// Заменяем шаблоны вида {#key} на переменные окружения
fn replace_file_variables(command: &str, variables: &HashMap<String, String>) -> String {
    let mut result = command.to_string();
    
    for (key, value) in variables {
        let placeholder = format!("{{#{}}}", key);
        result = result.replace(&placeholder, value);
    }
    
    result
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Создание тестового файла с переменными
    let test_file = "test_vars.json";
    if !Path::new(test_file).exists() {
        let test_vars = r#"{"TEST_VAR": "test_value", "ANOTHER_VAR": "another_value"}"#;
        fs::write(test_file, test_vars)?;
        println!("Создан тестовый файл {}", test_file);
    }
    
    // Загружаем переменные из файла
    let variables = match load_variables_from_file(test_file) {
        Ok(vars) => {
            println!("Успешно загружены переменные из файла:");
            for (key, value) in &vars {
                println!("  {}: {}", key, value);
            }
            vars
        },
        Err(e) => {
            println!("Ошибка загрузки переменных: {}", e);
            HashMap::new()
        }
    };
    
    // Тест 1: проверка обработки {#VAR} через variables_file (ручная реализация)
    println!("\nТест 1: Проверка обработки {{#VAR}} (ручная реализация)");
    let command_str = "echo 'Переменная из файла: {#TEST_VAR}, другая переменная: {#ANOTHER_VAR}'";
    let replaced_command = replace_file_variables(command_str, &variables);
    
    println!("Исходная команда: {}", command_str);
    println!("Команда с подстановкой: {}", replaced_command);
    
    let command = CommandBuilder::new(
        "test_cmd_1",
        &replaced_command
    )
    .execution_mode(ExecutionMode::Sequential)
    .build();
    
    let result = command.execute().await?;
    println!("Результат выполнения: {}", result.output.trim());
    
    // Тест 2: проверка комбинации с интерактивными переменными
    println!("\nТест 2: Проверка комбинации с интерактивными переменными");
    let command_str = "echo 'Из файла: {#TEST_VAR}, интерактивная: {INTERACTIVE_VAR}'";
    let replaced_command = replace_file_variables(command_str, &variables);
    
    let mut builder = CommandBuilder::new(
        "test_cmd_2",
        &replaced_command
    )
    .execution_mode(ExecutionMode::Sequential);
    
    // Добавляем предустановленные значения для интерактивных переменных
    builder = builder.env_var("INTERACTIVE_VAR", "значение для интерактивной переменной");
    
    let command = builder.build();
    let result = command.execute().await?;
    println!("Результат выполнения: {}", result.output.trim());
    
    // Тест 3: проверка комбинации с переменными окружения
    println!("\nТест 3: Проверка комбинации с переменными окружения");
    
    // Устанавливаем переменную окружения
    env::set_var("ENV_TEST_VAR", "значение из переменной окружения");
    
    let command_str = "echo 'Из файла: {#TEST_VAR}, из окружения: $ENV_TEST_VAR'";
    let replaced_command = replace_file_variables(command_str, &variables);
    
    let command = CommandBuilder::new(
        "test_cmd_3",
        &replaced_command
    )
    .execution_mode(ExecutionMode::Sequential)
    .build();
    
    let result = command.execute().await?;
    println!("Результат выполнения: {}", result.output.trim());
    
    // Тест 4: проверка с отсутствующей переменной в файле
    println!("\nТест 4: Проверка с отсутствующей переменной в файле");
    let command_str = "echo 'Отсутствующая переменная: {#MISSING_VAR}'";
    let replaced_command = replace_file_variables(command_str, &variables);
    
    let command = CommandBuilder::new(
        "test_cmd_4",
        &replaced_command
    )
    .execution_mode(ExecutionMode::Sequential)
    .build();
    
    let result = command.execute().await?;
    println!("Результат выполнения: {}", result.output.trim());
    
    Ok(())
} 