/*!
# Модуль Executor

Модуль `executor` отвечает за выполнение цепочек команд деплоя:

- Настройка и запуск цепочек команд с заданными параметрами
- Обработка рабочих директорий и переменных окружения
- Обработка ошибок и выполнение команд отката
- Журналирование процесса выполнения и результатов
- Поддержка шаблонных переменных в командах
- Использование переменных из окружения и файлов
- Поддержка глобальных переменных из settings.json

## Структура модуля

- `command_executor` - выполнение команд через SystemCommand
- `chain_builder` - создание и настройка цепочек команд
- `runner` - запуск цепочек команд с обработкой ошибок и откатом
- `variable_loader` - обработка переменных при выполнении команд

## Основные функции

- `run_commands` - запускает выполнение цепочки команд для указанного деплоя и события
- `execute_command_with_variables` - выполняет команду с подстановкой переменных
*/

pub mod chain_builder;
pub mod command_executor;
pub mod runner;
pub mod variable_loader;

use crate::config::Config;
use anyhow::Result;
use std::collections::HashMap;

/// Проверяет существование и создает рабочую директорию при необходимости
#[allow(dead_code)]
pub fn setup_working_directory(path: &str) -> Result<()> {
    runner::setup_working_directory(path)
}

/// Создает команду с заданными параметрами
#[allow(dead_code)]
pub fn create_command(
    name: &str,
    command: &str,
    working_dir: Option<&str>,
    env_vars: &[(String, String)],
    rollback_command: Option<&str>,
) -> command_system::command::ShellCommand {
    command_executor::create_simple_command(name, command, working_dir, env_vars, rollback_command)
}

/// Создает команду с поддержкой переменных
#[allow(dead_code)]
pub fn create_command_with_variables(
    name: &str,
    command: &str,
    working_dir: Option<&str>,
    env_vars: &[(String, String)],
    rollback_command: Option<&str>,
    variables: Option<HashMap<String, String>>,
    variables_file: Option<&str>,
    global_variables_file: Option<&str>,
) -> command_system::command::ShellCommand {
    command_executor::create_command(
        name,
        command,
        working_dir,
        env_vars,
        rollback_command,
        true,
        variables,
        variables_file,
        global_variables_file,
    )
}

/// Строит цепочку команд для указанного деплоя и события
#[allow(dead_code)]
pub fn build_command_chain(
    config: &Config,
    deployment_name: &str,
    event_name: &str,
    global_variables_file: Option<&str>,
) -> Result<command_system::chain::CommandChain> {
    chain_builder::build_command_chain(config, deployment_name, event_name, global_variables_file)
}

/// Запускает выполнение команд для деплоя и события
#[allow(dead_code)]
pub async fn run_commands(
    config: &Config,
    deployment_name: &str,
    event_name: &str,
    global_variables_file: Option<&str>,
) -> Result<()> {
    let history_path = runner::get_history_path();
    runner::execute_command(config, deployment_name, event_name, &history_path).await
}
