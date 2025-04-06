use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(
    name = "deploy-commander",
    version,
    about = "Утилита для выполнения команд при деплое"
)]
pub struct Cli {
    /// Путь к файлу конфигурации
    #[clap(short, long, default_value = "deploy-config.yml")]
    pub config: String,

    /// Подробный вывод информации
    #[clap(short, long)]
    pub verbose: bool,

    /// Путь к файлу журнала
    #[clap(long, default_value = "deploy-commander.log")]
    pub log_file: String,

    /// Команда для выполнения
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Запустить команды для указанного деплоя и события
    Run {
        /// Название деплоя
        #[clap(short, long)]
        deployment: String,

        /// Название события
        #[clap(short, long)]
        event: String,
    },

    /// Вывести список доступных деплоев и событий
    List,

    /// Создать новый шаблон деплоя
    Create {
        /// Название нового деплоя
        #[clap(short, long)]
        deployment: String,
    },

    /// Проверить конфигурацию деплоя
    Verify {
        /// Название деплоя для проверки
        #[clap(short, long)]
        deployment: String,
    },

    /// Показать историю деплоев
    History {
        /// Название деплоя для просмотра истории
        #[clap(short, long)]
        deployment: String,

        /// Количество последних записей для отображения
        #[clap(short, long, default_value = "10")]
        limit: usize,
    },

    /// Очистить историю деплоев
    ClearHistory {
        /// Название деплоя для очистки истории (если не указано, очищается вся история)
        #[clap(short, long)]
        deployment: Option<String>,
    },
}
