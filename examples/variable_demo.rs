// Пример использования шаблонных переменных
// Пример демонстрирует различные способы использования переменных в командах:
// 1. Интерактивный ввод
// 2. Переменные окружения
// 3. Переменные из файлов
// 4. Переменные из глобального файла

use std::collections::HashMap;
use std::env;
use std::fs;

fn main() {
    // Путь к файлу переменных
    let variables_file = "examples/variables.json";
    let global_variables_file = "variables.json";

    // Создаем файл переменных, если его нет
    create_variables_file(variables_file);

    // Читаем переменные из файла
    let file_variables = read_variables_from_file(variables_file);
    println!("Переменные из файла:");
    for (key, value) in &file_variables {
        println!("  {}: {}", key, value);
    }

    // Читаем глобальные переменные
    let global_variables = read_variables_from_file(global_variables_file);
    println!("\nГлобальные переменные:");
    for (key, value) in &global_variables {
        println!("  {}: {}", key, value);
    }

    // Примеры команд с переменными
    println!("\nПримеры команд с переменными:");

    // 1. Простая команда с интерактивным вводом
    let command = "echo 'Привет, {name}!'";
    println!("Команда: {}", command);
    println!("  -> Замена: Привет, <запрос имени пользователя>!");

    // 2. Команда с переменной окружения
    let command = "echo 'Среда: {$NODE_ENV}'";
    println!("Команда: {}", command);
    env::set_var("NODE_ENV", "production");
    println!("  -> Замена: Среда: production");

    // 3. Команда с переменной из файла
    let command = "echo 'Версия: {#VERSION}'";
    println!("Команда: {}", command);
    println!(
        "  -> Замена: Версия: {}",
        file_variables
            .get("VERSION")
            .unwrap_or(&"не найдена".to_string())
    );

    // 4. Команда с глобальной переменной
    let command = "echo 'Глобальная версия: {#GLOBAL_VERSION}'";
    println!("Команда: {}", command);
    println!(
        "  -> Замена: Глобальная версия: {}",
        global_variables
            .get("GLOBAL_VERSION")
            .unwrap_or(&"не найдена".to_string())
    );

    // 5. Смешанные переменные
    let command = "echo 'Проект {project_name} версии {#VERSION} запущен в {$NODE_ENV} окружении на {#GLOBAL_DATABASE_HOST}:{#GLOBAL_DATABASE_PORT}'";
    println!("Команда: {}", command);
    println!("  -> Замена: Проект <запрос имени проекта> версии {} запущен в production окружении на {}:{}",
        file_variables.get("VERSION").unwrap_or(&"не найдена".to_string()),
        global_variables.get("GLOBAL_DATABASE_HOST").unwrap_or(&"не найден".to_string()),
        global_variables.get("GLOBAL_DATABASE_PORT").unwrap_or(&"не найден".to_string())
    );

    // 6. Приоритет локальных переменных над глобальными
    let command = "echo 'Версия API: {#API_URL}'";
    println!("Команда: {}", command);
    println!(
        "  -> Замена: Версия API: {}",
        file_variables.get("API_URL").unwrap_or(
            global_variables
                .get("GLOBAL_API_URL")
                .unwrap_or(&"не найдена".to_string())
        )
    );

    println!("\nВ реальной работе Deploy Commander:");
    println!("1. Для интерактивных переменных будет запрошен ввод");
    println!("2. Для переменных окружения ($) будут подставлены значения из окружения");
    println!("3. Для переменных из файла (#) будут подставлены значения из указанного файла или глобального файла");
    println!("4. Локальные переменные имеют приоритет над глобальными");
}

fn create_variables_file(file_path: &str) {
    let content = r#"{
    "VERSION": "2.5.0",
    "API_URL": "https://api.local.example.com",
    "DB_HOST": "127.0.0.1",
    "DB_PORT": "3306",
    "DEBUG": "true",
    "APP_NAME": "ExampleApp"
}"#;

    fs::write(file_path, content).expect("Не удалось записать файл переменных");
}

fn read_variables_from_file(file_path: &str) -> HashMap<String, String> {
    let mut variables = HashMap::new();

    match fs::read_to_string(file_path) {
        Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(json) => {
                if let serde_json::Value::Object(map) = json {
                    for (key, value) in map {
                        if let Some(string_value) = value.as_str() {
                            variables.insert(key.clone(), string_value.to_string());
                        } else {
                            variables.insert(key.clone(), value.to_string());
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Ошибка разбора JSON: {}", e);
            }
        },
        Err(e) => {
            eprintln!("Ошибка чтения файла {}: {}", file_path, e);
        }
    }

    variables
}
