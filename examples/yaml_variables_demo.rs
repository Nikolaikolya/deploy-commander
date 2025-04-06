use anyhow::Result;
use command_system::{CommandBuilder, CommandExecution};
use log::LevelFilter;
use std::env;

/// Пример демонстрирует загрузку переменных из YAML файла
#[tokio::main]
async fn main() -> Result<()> {
    // Настраиваем логгирование
    simple_logger::SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init()
        .unwrap();

    println!("\n=== ДЕМОНСТРАЦИЯ ЗАГРУЗКИ ПЕРЕМЕННЫХ ИЗ YAML ФАЙЛА ===\n");

    // Устанавливаем переменную окружения для теста
    env::set_var("USERNAME", "test_user");
    env::set_var("DB_HOST", "env_host");
    env::set_var("DB_PORT", "1234");

    // Пример 1: Загрузка простых значений из YAML файла
    println!("Тест 1: Загрузка простых значений из YAML файла");
    let cmd = CommandBuilder::new(
        "yaml_test_1",
        "echo 'DB: {#DB_HOST}:{#DB_PORT}, API Key: {#API_KEY}'"
    )
    .variables_file("./examples/variables.yml")
    .build();
    
    let result = cmd.execute().await?;

    println!("[ОТЛАДКА] Результат выполнения команды:");
    println!("[ОТЛАДКА] - success: {}", result.success);
    println!("[ОТЛАДКА] - output (raw): {:?}", result.output);
    println!("[ОТЛАДКА] - error: {:?}", result.error);
    println!("[ОТЛАДКА] - output (строка): {:?}", result.output.trim());
    println!("Результат загрузки простых значений из YAML: {}\n", result.output.trim());

    // Пример 2: Загрузка сложных значений из YAML файла
    println!("Тест 2: Загрузка сложных значений из YAML файла");
    let cmd = CommandBuilder::new(
        "yaml_test_2",
        "echo 'Сложное значение: {#complex_value}'"
    )
    .variables_file("./examples/variables.yml")
    .build();
    
    let result = cmd.execute().await?;

    println!("[ОТЛАДКА] Результат выполнения команды:");
    println!("[ОТЛАДКА] - success: {}", result.success);
    println!("[ОТЛАДКА] - output (raw): {:?}", result.output);
    println!("[ОТЛАДКА] - error: {:?}", result.error);
    println!("[ОТЛАДКА] - output (строка): {:?}", result.output.trim());
    println!("Результат загрузки сложного значения: {}\n", result.output.trim());
    
    // Пример 3: Загрузка переменной с подстановкой из окружения
    println!("Тест 3: Загрузка переменной с подстановкой из окружения");
    let cmd = CommandBuilder::new(
        "yaml_test_3",
        "echo 'Строка подключения: {#CONNECTION_STRING}'"
    )
    .variables_file("./examples/variables.yml")
    .build();
    
    let result = cmd.execute().await?;

    println!("[ОТЛАДКА] Результат выполнения команды:");
    println!("[ОТЛАДКА] - success: {}", result.success);
    println!("[ОТЛАДКА] - output (raw): {:?}", result.output);
    println!("[ОТЛАДКА] - error: {:?}", result.error);
    println!("[ОТЛАДКА] - output (строка): {:?}", result.output.trim());
    println!("Результат загрузки переменной с подстановкой: {}\n", result.output.trim());
    
    // Пример 4: Загрузка многострочного значения из YAML файла
    println!("Тест 4: Загрузка многострочного значения из YAML файла");
    let cmd = CommandBuilder::new(
        "yaml_test_4",
        "echo 'Конфигурация сервера: \n{#SERVER_CONFIG}'"
    )
    .variables_file("./examples/variables.yml")
    .build();
    
    let result = cmd.execute().await?;

    println!("[ОТЛАДКА] Результат выполнения команды:");
    println!("[ОТЛАДКА] - success: {}", result.success);
    println!("[ОТЛАДКА] - output (строка): {}", result.output.trim());
    println!("Результат загрузки многострочного значения: см. выше\n");
    
    // Пример 5: Сравнение приоритета переменных из JSON и YAML
    println!("Тест 5: Сравнение приоритета переменных из JSON и YAML");
    
    // Сначала загружаем из JSON (меньший приоритет)
    let cmd = CommandBuilder::new(
        "priority_test",
        "echo 'API ключ (JSON > YAML): {#API_KEY}, DB хост (JSON > YAML): {#DB_HOST}'"
    )
    .variables_file("./examples/variables.json")  // Загружаем сначала JSON (меньший приоритет)
    .variables_file("./examples/variables.yml")   // Затем YAML (больший приоритет)
    .build();
    
    let result = cmd.execute().await?;

    println!("[ОТЛАДКА] Результат выполнения команды (JSON затем YAML):");
    println!("[ОТЛАДКА] - output (строка): {}", result.output.trim());
    
    // Затем загружаем из YAML (меньший приоритет)
    let cmd = CommandBuilder::new(
        "priority_test_reversed",
        "echo 'API ключ (YAML > JSON): {#API_KEY}, DB хост (YAML > JSON): {#DB_HOST}'"
    )
    .variables_file("./examples/variables.yml")   // Загружаем сначала YAML (меньший приоритет)
    .variables_file("./examples/variables.json")  // Затем JSON (больший приоритет)
    .build();
    
    let result = cmd.execute().await?;

    println!("[ОТЛАДКА] Результат выполнения команды (YAML затем JSON):");
    println!("[ОТЛАДКА] - output (строка): {}", result.output.trim());
    println!("Результат теста приоритета переменных: значения меняются в зависимости от порядка загрузки\n");

    println!("Все тесты завершены");
    
    Ok(())
} 