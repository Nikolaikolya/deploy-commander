use std::collections::HashMap;
use anyhow::Result;
use serde_yaml::Value;

fn main() -> Result<()> {
    println!("=== ТЕСТ ЗАГРУЗКИ YAML ПЕРЕМЕННЫХ ===");
    
    // Создаем тестовый YAML
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
    
    println!("Исходный YAML:\n{}", yaml_content);
    
    // Парсим в serde_yaml::Value
    let yaml: Value = serde_yaml::from_str(yaml_content)?;
    println!("\nРазобранное значение YAML:\n{:?}", yaml);
    
    // Преобразуем в карту строк
    let mut vars = HashMap::new();
    if let Value::Mapping(map) = yaml {
        for (key, value) in map {
            if let Some(key_str) = key.as_str() {
                let value_str = match value {
                    Value::String(s) => s,
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Sequence(_) | Value::Mapping(_) => {
                        match serde_json::to_string(&value) {
                            Ok(s) => s,
                            Err(_) => format!("{:?}", value),
                        }
                    },
                    _ => format!("{:?}", value),
                };
                vars.insert(key_str.to_string(), value_str);
            }
        }
    }
    
    println!("\nПреобразованные переменные:");
    for (key, value) in &vars {
        println!("{} = {}", key, value);
    }
    
    // Проверяем доступ к сложному значению
    if let Some(complex) = vars.get("complex_value") {
        println!("\nСложное значение: {}", complex);
    }
    
    Ok(())
} 