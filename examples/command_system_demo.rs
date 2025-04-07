// Пример работы с библиотекой command_system
// Показывает как использовать CommandBuilder, variables_file, и обработку результатов

use command_system::{CommandBuilder, CommandExecution};
use std::env;
use std::fs;

#[tokio::main]
async fn main() {
    // Создаем тестовый файл с переменными
    create_test_vars_file();

    println!("=== Пример работы с библиотекой command_system ===\n");

    // Пример 1: Простая команда без переменных
    println!("=== Простая команда без переменных ===");
    let command = CommandBuilder::new("simple_test", "echo 'Привет, мир!'").build();

    match command.execute().await {
        Ok(result) => {
            println!("Успех: {}", result.success);
            println!("Вывод: {}", result.output);
            println!("Длительность: {} мс", result.duration_ms);
        }
        Err(e) => {
            println!("Ошибка: {}", e);
        }
    }

    // Пример 2: Команда с интерактивными переменными
    println!("\n=== Команда с интерактивными переменными ===");
    let command = CommandBuilder::new("interactive_test", "echo 'Привет, {name}!'").build();

    match command.execute().await {
        Ok(result) => {
            println!("Успех: {}", result.success);
            println!("Вывод: {}", result.output);
            println!("Длительность: {} мс", result.duration_ms);
        }
        Err(e) => {
            println!("Ошибка: {}", e);
        }
    }

    // Пример 3: Команда с переменными окружения
    println!("\n=== Команда с переменными окружения ===");
    env::set_var("TEST_NAME", "Пользователь из ENV");

    let command = CommandBuilder::new("env_test", "echo 'Привет, {$TEST_NAME}!'").build();

    match command.execute().await {
        Ok(result) => {
            println!("Успех: {}", result.success);
            println!("Вывод: {}", result.output);
            println!("Длительность: {} мс", result.duration_ms);
        }
        Err(e) => {
            println!("Ошибка: {}", e);
        }
    }

    // Пример 4: Команда с переменными из файла
    println!("\n=== Команда с переменными из файла ===");
    let command = CommandBuilder::new("file_test", "echo 'Привет, {#FILE_SYSTEM}!'")
        .variables_file("test_vars.json")
        .build();

    match command.execute().await {
        Ok(result) => {
            println!("Успех: {}", result.success);
            println!("Вывод: {}", result.output);
            println!("Длительность: {} мс", result.duration_ms);
        }
        Err(e) => {
            println!("Ошибка: {}", e);
        }
    }

    // Пример 5: Смешанный вариант всех типов переменных
    println!("\n=== Смешанный вариант ===");
    let command = CommandBuilder::new(
        "mixed_test",
        "echo 'Интерактивно: {interactive_var}, из env: {$TEST_NAME}, из файла: {#FILE_SYSTEM}'",
    )
    .variables_file("test_vars.json")
    .build();

    match command.execute().await {
        Ok(result) => {
            println!("Успех: {}", result.success);
            println!("Вывод: {}", result.output);
            println!("Длительность: {} мс", result.duration_ms);

            // Демонстрация полей CommandResult
            println!("\nПоля CommandResult:");
            println!("id: {}", result.id);
            println!("command_name: {}", result.command_name);
            println!("success: {}", result.success);
            println!("output: {}", result.output);
            println!("error: {:?}", result.error);
            println!("exit_code: {:?}", result.exit_code);
            println!("start_time: {}", result.start_time);
            println!("end_time: {}", result.end_time);
            println!("duration_ms: {}", result.duration_ms);
        }
        Err(e) => {
            println!("Ошибка: {}", e);
        }
    }

    // Удаляем тестовый файл
    fs::remove_file("test_vars.json").ok();
}

fn create_test_vars_file() {
    let content = r#"{
    "FILE_SYSTEM": "Значение из файла"
}"#;

    fs::write("test_vars.json", content).expect("Не удалось создать тестовый файл с переменными");
}
