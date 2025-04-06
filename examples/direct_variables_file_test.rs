use std::fs;
use std::path::Path;
use std::env;
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
    
    // Устанавливаем переменную окружения для теста
    env::set_var("ENV_TEST_VAR", "значение из переменной окружения");
    
    println!("\n=== ПРЯМОЕ ТЕСТИРОВАНИЕ VARIABLES_FILE В COMMANDBUILDER ===\n");
    
    // Тест 1: Прямое использование variables_file в CommandBuilder
    println!("\nТест 1: Прямое использование variables_file в CommandBuilder");
    
    let command = CommandBuilder::new(
        "test_file_vars",
        "echo 'Переменная из файла: {#TEST_VAR}, другая переменная: {#ANOTHER_VAR}'"
    )
    .execution_mode(ExecutionMode::Sequential)
    .variables_file(test_vars_path)
    .build();
    
    let result = command.execute().await?;
    println!("[ОТЛАДКА] Результат выполнения команды:");
    println!("[ОТЛАДКА] - success: {}", result.success);
    println!("[ОТЛАДКА] - output (raw): {:?}", result.output);
    println!("[ОТЛАДКА] - error: {:?}", result.error);
    println!("[ОТЛАДКА] - output (строка): \"{}\"", result.output.trim());
    println!("Результат с переменными из файла: {}", result.output.trim());
    
    // Тест 2: Переменная из файла с отсутствующим значением
    println!("\nТест 2: Переменная из файла с отсутствующим значением");
    
    let command = CommandBuilder::new(
        "test_missing_var",
        "echo 'Отсутствует в файле: {#MISSING_VAR}'"
    )
    .execution_mode(ExecutionMode::Sequential)
    .variables_file(test_vars_path)
    .build();
    
    let result = command.execute().await?;
    println!("[ОТЛАДКА] Результат выполнения команды:");
    println!("[ОТЛАДКА] - success: {}", result.success);
    println!("[ОТЛАДКА] - output (raw): {:?}", result.output);
    println!("[ОТЛАДКА] - error: {:?}", result.error);
    println!("[ОТЛАДКА] - output (строка): \"{}\"", result.output.trim());
    println!("Результат с отсутствующей переменной: {}", result.output.trim());
    
    // Тест 3: Переменная окружения
    println!("\nТест 3: Переменная окружения");
    
    let command = CommandBuilder::new(
        "test_env_var",
        "echo 'Из окружения: {$ENV_TEST_VAR}'"
    )
    .execution_mode(ExecutionMode::Sequential)
    .variables_file(test_vars_path)
    .build();
    
    let result = command.execute().await?;
    println!("[ОТЛАДКА] Результат выполнения команды:");
    println!("[ОТЛАДКА] - success: {}", result.success);
    println!("[ОТЛАДКА] - output (raw): {:?}", result.output);
    println!("[ОТЛАДКА] - error: {:?}", result.error);
    println!("[ОТЛАДКА] - output (строка): \"{}\"", result.output.trim());
    println!("Результат с переменной окружения: {}", result.output.trim());
    
    // Тест 4: Пример из вопроса пользователя
    println!("\nТест 4: Пример из вопроса пользователя");
    
    let command = CommandBuilder::new(
        "missing_test",
        "echo 'Отсутствует в env: {$MISSING_VAR}, отсутствует в файле: {#MISSING_VAR}'"
    )
    .execution_mode(ExecutionMode::Sequential)
    .variables_file(test_vars_path)
    .build();
    
    let result = command.execute().await?;
    println!("[ОТЛАДКА] Результат выполнения команды:");
    println!("[ОТЛАДКА] - success: {}", result.success);
    println!("[ОТЛАДКА] - output (raw): {:?}", result.output);
    println!("[ОТЛАДКА] - error: {:?}", result.error);
    println!("[ОТЛАДКА] - output (строка): \"{}\"", result.output.trim());
    println!("Результат с отсутствующими переменными: {}", result.output.trim());
    
    println!("Тест 5: Проверка вывода результатов без println");
    // Строим команду, которая возвращает результат, но мы его не выводим в println
    let command = CommandBuilder::new("direct_test_5", "echo 'Вывод без println'")
        .variables_file("test_vars.json")
        .build();
    
    let _result = command.execute().await?; // Намеренно не используем println

    println!("\nТест 6: Проверка вывода с трассировкой");
    let command = CommandBuilder::new("direct_test_6", "echo 'Трассировка вывода'")
        .build();
    
    let result = command.execute().await?;
    println!("[ТРАССИРОВКА] Полная информация о выполнении команды:");
    println!("  Имя команды: direct_test_6");
    println!("  Успех: {}", result.success);
    println!("  Вывод (строка): '{}'", result.output);
    println!("  Вывод (байты): {:?}", result.output.as_bytes());
    println!("  Ошибка: {:?}", result.error);
    
    Ok(())
} 