use std::fs;
use std::path::Path;
use std::env;
use std::process::Command as StdCommand;
use tokio;
use anyhow::Result;
use command_system::{CommandBuilder, CommandExecution, ExecutionMode};

#[tokio::main]
async fn main() -> Result<()> {
    // Создаем тестовый файл с переменными, если он не существует
    let test_vars_path = "test_vars.json";
    if !Path::new(test_vars_path).exists() {
        let test_vars = r#"{"TEST_VAR": "test_value", "ANOTHER_VAR": "another_value"}"#;
        fs::write(test_vars_path, test_vars)?;
        println!("Создан тестовый файл {}", test_vars_path);
    }
    
    // Проверяем вывод стандартной команды напрямую
    println!("\n=== Проверка вывода стандартной команды напрямую ===");
    let std_output = StdCommand::new("echo")
        .arg("Тест вывода через std::process::Command")
        .output()?;
    
    println!("Стандартный вывод: {}", String::from_utf8_lossy(&std_output.stdout));
    println!("Статус: {}", std_output.status);
    
    // Проверка через CommandBuilder без variables_file
    println!("\n=== Проверка вывода через CommandBuilder без variables_file ===");
    let command = CommandBuilder::new(
        "simple_test",
        "echo 'Простой тест без переменных'"
    )
    .execution_mode(ExecutionMode::Sequential)
    .build();
    
    let result = command.execute().await?;
    println!("Вывод через CommandBuilder:");
    println!("success: {}", result.success);
    println!("output: {:?}", result.output);
    println!("error: {:?}", result.error);
    
    // Проверка с variables_file
    println!("\n=== Проверка вывода через CommandBuilder с variables_file ===");
    let command = CommandBuilder::new(
        "vars_test",
        "echo 'Переменная из файла: {#TEST_VAR}'"
    )
    .execution_mode(ExecutionMode::Sequential)
    .variables_file(test_vars_path)
    .build();
    
    let result = command.execute().await?;
    println!("Вывод через CommandBuilder с variables_file:");
    println!("success: {}", result.success);
    println!("output: {:?}", result.output);
    println!("error: {:?}", result.error);
    
    // Проверка подставлены ли значения в команду
    println!("\n=== Подробная проверка command.execute ===");
    let command_str = "echo 'Явная переменная: TEST_VALUE'";
    println!("Исходная команда: {}", command_str);
    
    let command = CommandBuilder::new(
        "explicit_test",
        command_str
    )
    .execution_mode(ExecutionMode::Sequential)
    .build();
    
    let result = command.execute().await?;
    println!("Результат:");
    println!("success: {}", result.success);
    println!("output в кавычках: \"{:?}\"", result.output);
    println!("Длина вывода: {}", result.output.len());
    println!("Байты вывода: {:?}", result.output.as_bytes());
    
    // Проверка вывода команды с пайпом
    println!("\n=== Проверка вывода с пайпом ===");
    let command = CommandBuilder::new(
        "pipe_test",
        "echo 'Тест с пайпом' | cat"
    )
    .execution_mode(ExecutionMode::Sequential)
    .build();
    
    let result = command.execute().await?;
    println!("Результат с пайпом:");
    println!("success: {}", result.success);
    println!("output: {:?}", result.output);
    
    Ok(())
} 