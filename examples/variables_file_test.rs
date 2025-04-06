use std::fs;
use std::path::Path;
use std::env;
use command_system::{CommandBuilder, CommandExecution, ExecutionMode};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Создаем тестовый файл с переменными, если он не существует
    let test_vars_path = "test_vars.json";
    if !Path::new(test_vars_path).exists() {
        let test_vars = r#"{"TEST_VAR": "test_value", "ANOTHER_VAR": "another_value"}"#;
        fs::write(test_vars_path, test_vars)?;
        println!("Создан тестовый файл {}", test_vars_path);
    }
    
    // Устанавливаем переменную окружения для теста
    env::set_var("ENV_TEST_VAR", "значение из переменной окружения");
    
    // Тест 1: Проверка variables_file с {#VAR}
    println!("\nТест 1: Проверка variables_file с {{#VAR}}");
    let command = CommandBuilder::new(
        "test_file_vars",
        "echo 'Переменная из файла: {#TEST_VAR}, другая переменная: {#ANOTHER_VAR}'"
    )
    .execution_mode(ExecutionMode::Sequential)
    .variables_file(test_vars_path)
    .build();
    
    let result = command.execute().await?;
    println!("Результат с переменными из файла: {}", result.output.trim());
    
    // Тест 2: Проверка variables_file с отсутствующей переменной
    println!("\nТест 2: Проверка variables_file с отсутствующей переменной");
    let command = CommandBuilder::new(
        "test_missing_var",
        "echo 'Отсутствует в файле: {#MISSING_VAR}'"
    )
    .execution_mode(ExecutionMode::Sequential)
    .variables_file(test_vars_path)
    .build();
    
    let result = command.execute().await?;
    println!("Результат с отсутствующей переменной: {}", result.output.trim());
    
    // Тест 3: Проверка переменных окружения {$VAR}
    println!("\nТест 3: Проверка переменных окружения {{$VAR}}");
    let command = CommandBuilder::new(
        "test_env_vars",
        "echo 'Переменная из окружения: {$ENV_TEST_VAR}'"
    )
    .execution_mode(ExecutionMode::Sequential)
    .variables_file(test_vars_path)
    .build();
    
    let result = command.execute().await?;
    println!("Результат с переменной окружения: {}", result.output.trim());
    
    // Тест 4: Проверка обычных интерактивных переменных {VAR}
    println!("\nТест 4: Проверка обычных интерактивных переменных {{VAR}}");
    let command = CommandBuilder::new(
        "test_interactive_vars",
        "echo 'Интерактивная переменная: {INTERACTIVE_VAR}'"
    )
    .execution_mode(ExecutionMode::Sequential)
    .variables_file(test_vars_path)
    .build();
    
    let result = command.execute().await?;
    println!("Результат с интерактивной переменной: {}", result.output.trim());
    
    // Тест 5: Проверка комбинации всех типов переменных
    println!("\nТест 5: Проверка комбинации всех типов переменных");
    let command = CommandBuilder::new(
        "test_combined_vars",
        "echo 'Из файла: {#TEST_VAR}, из окружения: {$ENV_TEST_VAR}, интерактивная: {INTERACTIVE_VAR}'"
    )
    .execution_mode(ExecutionMode::Sequential)
    .variables_file(test_vars_path)
    .build();
    
    let result = command.execute().await?;
    println!("Результат с комбинацией переменных: {}", result.output.trim());
    
    // Тест 6: Пример из вопроса пользователя
    println!("\nТест 6: Пример из вопроса пользователя");
    let command = CommandBuilder::new(
        "missing_test",
        "echo 'Отсутствует в env: {$MISSING_VAR}, отсутствует в файле: {#MISSING_VAR}'"
    )
    .variables_file("test_vars.json")
    .build();
    
    let result = command.execute().await?;
    println!("Результат с отсутствующими переменными: {}", result.output.trim());
    
    Ok(())
} 