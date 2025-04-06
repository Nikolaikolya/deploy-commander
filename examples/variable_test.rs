use command_system::{CommandBuilder, CommandExecution, ExecutionMode};
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Создаем тестовый файл с переменными, если он не существует
    if !Path::new("test_vars.json").exists() {
        let test_vars = r#"{"TEST_VAR": "test_value", "ANOTHER_VAR": "another_value"}"#;
        fs::write("test_vars.json", test_vars)?;
        println!("Создан тестовый файл test_vars.json");
    }
    
    println!("Тест 1: Тестирование CommandBuilder с variables_file");
    let command = CommandBuilder::new(
        "variables_test",
        "echo 'Значение TEST_VAR: {#TEST_VAR}, значение ANOTHER_VAR: {#ANOTHER_VAR}'"
    )
    .execution_mode(ExecutionMode::Sequential)
    .variables_file("test_vars.json")
    .build();
    
    let result = command.execute().await?;
    println!("Результат выполнения команды:");
    println!("{}", result.output);
    
    println!("\nТест 2: Проверка отсутствующих переменных");
    let command = CommandBuilder::new(
        "missing_test",
        "echo 'Отсутствует в файле: {#MISSING_VAR}'"
    )
    .execution_mode(ExecutionMode::Sequential)
    .variables_file("test_vars.json")
    .build();
    
    let result = command.execute().await?;
    println!("Результат выполнения команды:");
    println!("{}", result.output);
    
    println!("\nТест 3: Проверка несуществующего файла переменных");
    let command = CommandBuilder::new(
        "nonexistent_file_test",
        "echo 'Тест с несуществующим файлом переменных'"
    )
    .execution_mode(ExecutionMode::Sequential)
    .variables_file("nonexistent_file.json")
    .build();
    
    let result = command.execute().await?;
    println!("Результат выполнения команды:");
    println!("{}", result.output);
    
    Ok(())
} 