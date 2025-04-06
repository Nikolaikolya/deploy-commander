use std::fs;
use std::path::Path;
use std::collections::HashMap;
use command_system::{CommandBuilder, CommandExecution, ExecutionMode};

// Чтение переменных из JSON-файла
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
    
    Ok(vars)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Проверка работы с файлом переменных в CommandBuilder");
    
    // Создаем тестовый файл с переменными, если он не существует
    let test_vars_path = "test_vars.json";
    if !Path::new(test_vars_path).exists() {
        let test_vars = r#"{"TEST_VAR": "test_value", "ANOTHER_VAR": "another_value"}"#;
        fs::write(test_vars_path, test_vars)?;
        println!("Создан тестовый файл {}", test_vars_path);
    }
    
    // Тест 1: Проверка переменных из файла
    println!("\nТест 1: Загрузка и отображение переменных из файла");
    match load_variables_from_file(test_vars_path) {
        Ok(vars) => {
            println!("Успешно загружены переменные из файла:");
            for (key, value) in &vars {
                println!("  {}: {}", key, value);
            }
        },
        Err(e) => println!("Ошибка загрузки переменных: {}", e),
    }
    
    // Тест 2: Проверка использования шаблонов {key}
    println!("\nТест 2: Проверка использования шаблонов {{key}}");
    
    let vars = load_variables_from_file(test_vars_path)?;
    let mut builder = CommandBuilder::new(
        "test_template_vars",
        "echo 'Переменная из файла: {TEST_VAR}, другая переменная: {ANOTHER_VAR}'"
    )
    .execution_mode(ExecutionMode::Sequential);
    
    for (key, value) in vars {
        builder = builder.env_var(&key, &value);
    }
    
    let command = builder.build();
    let result = command.execute().await?;
    println!("Результат с шаблонами {{key}}: {}", result.output.trim());
    
    // Тест 3: Проверка использования переменных окружения в формате $VAR
    println!("\nТест 3: Проверка использования переменных окружения в формате $VAR");
    
    let vars = load_variables_from_file(test_vars_path)?;
    let mut builder = CommandBuilder::new(
        "test_env_vars",
        "echo 'Переменная из файла: $TEST_VAR, другая переменная: $ANOTHER_VAR'"
    )
    .execution_mode(ExecutionMode::Sequential);
    
    for (key, value) in vars {
        builder = builder.env_var(&key, &value);
    }
    
    let command = builder.build();
    let result = command.execute().await?;
    println!("Результат с переменными $VAR: {}", result.output.trim());
    
    // Тест 4: Проверка подстановки переменных в формате {#VAR} (специфично для deploy-commander)
    println!("\nТест 4: Проверка подстановки переменных в формате {{#VAR}}");
    
    let vars = load_variables_from_file(test_vars_path)?;
    let mut builder = CommandBuilder::new(
        "test_hash_vars",
        "echo 'Переменная из файла: {#TEST_VAR}, другая переменная: {#ANOTHER_VAR}'"
    )
    .execution_mode(ExecutionMode::Sequential);
    
    for (key, value) in vars {
        builder = builder.env_var(&key, &value);
    }
    
    let command = builder.build();
    let result = command.execute().await?;
    println!("Результат с шаблонами {{#key}}: {}", result.output.trim());
    
    // Тест 5: Проверка отсутствующей переменной
    println!("\nТест 5: Проверка отсутствующей переменной");
    let command = CommandBuilder::new(
        "missing_var_cmd",
        "echo 'Отсутствующие переменные: $MISSING_VAR, {MISSING_VAR}, {#MISSING_VAR}'"
    )
    .execution_mode(ExecutionMode::Sequential)
    .build();
    
    let result = command.execute().await?;
    println!("Результат с отсутствующими переменными: {}", result.output.trim());
    
    Ok(())
} 