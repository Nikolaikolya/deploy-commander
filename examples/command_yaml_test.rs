use anyhow::Result;
use command_system::{CommandBuilder, CommandExecution};
use log::LevelFilter;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Пример демонстрирует использование YAML файла с CommandBuilder
#[tokio::main]
async fn main() -> Result<()> {
    // Настраиваем логгирование
    simple_logger::SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init()
        .unwrap();

    println!("\n=== ДЕМОНСТРАЦИЯ ИСПОЛЬЗОВАНИЯ YAML В КОМАНДАХ ===\n");

    // Создаем временный YAML файл для теста
    let yaml_file_path = "./examples/temp/test_vars.yml";
    let parent_dir = Path::new(yaml_file_path).parent().unwrap();
    if !parent_dir.exists() {
        std::fs::create_dir_all(parent_dir)?;
    }
    
    let yaml_content = r#"
# Базовые переменные
DB_HOST: localhost
DB_PORT: 5432
API_KEY: "yaml-api-key-123"
YAML_SPECIFIC: "это значение только в YAML"

# Сложное значение
complex_value:
  nested: true
  count: 42
  items:
    - "первый"
    - "второй"
"#;
    
    let mut file = File::create(yaml_file_path)?;
    file.write_all(yaml_content.as_bytes())?;
    println!("Создан тестовый YAML файл: {}", yaml_file_path);
    
    // Создаем временный JSON файл для сравнения
    let json_file_path = "./examples/temp/test_vars.json";
    let json_content = r#"
{
  "DB_HOST": "json-db-host",
  "DB_PORT": "9999",
  "API_KEY": "json-api-key-456",
  "JSON_SPECIFIC": "это значение только в JSON"
}
"#;
    
    let mut file = File::create(json_file_path)?;
    file.write_all(json_content.as_bytes())?;
    println!("Создан тестовый JSON файл: {}", json_file_path);
    
    // Тест 1: Использование JSON файла
    println!("\nТест 1: Использование JSON файла");
    
    let command = CommandBuilder::new(
        "json_test_1",
        "echo 'JSON: DB: {#DB_HOST}:{#DB_PORT}, API Key: {#API_KEY}'"
    )
    .variables_file(json_file_path)
    .build();
    
    let result = command.execute().await?;
    println!("[ОТЛАДКА] Результат выполнения команды:");
    println!("[ОТЛАДКА] - success: {}", result.success);
    println!("[ОТЛАДКА] - output: {}", result.output.trim());
    
    // Создаем простой файл для теста с чтением напрямую
    let json_content = r#"{"TEST_VAR": "test_value", "ANOTHER_VAR": "another_value"}"#;
    let test_json_path = "./examples/temp/simple_test.json";
    let mut file = File::create(test_json_path)?;
    file.write_all(json_content.as_bytes())?;
    
    // Тест 2: Загрузка простого JSON напрямую
    println!("\nТест 2: Загрузка простого JSON напрямую");
    
    let command = CommandBuilder::new(
        "direct_json_test",
        "echo 'Прямой JSON: TEST_VAR={TEST_VAR}, ANOTHER_VAR={ANOTHER_VAR}'"
    )
    .variables_file(test_json_path)
    .build();
    
    let result = command.execute().await?;
    println!("[ОТЛАДКА] Результат выполнения команды:");
    println!("[ОТЛАДКА] - success: {}", result.success);
    println!("[ОТЛАДКА] - output: {}", result.output.trim());
    
    // Удаляем временные файлы
    std::fs::remove_file(yaml_file_path)?;
    std::fs::remove_file(json_file_path)?;
    std::fs::remove_file(test_json_path)?;
    println!("\nВременные файлы удалены, тесты завершены");
    
    Ok(())
} 