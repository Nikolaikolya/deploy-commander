use log::{error, info, warn};
use std::process::exit;
use std::sync::{Arc, Mutex};

use crate::config::Config;
use crate::run::command_runner;
use crate::run::deployments;
use crate::run::history;
use crate::storage;

/// Структура для параметров выполнения события
pub struct EventExecutionParams<'a> {
    pub config: &'a Config,
    pub deployment_name: &'a str,
    pub event_name: &'a str,
    pub history_path: &'a str,
}

/// Структура для параметров выполнения всех событий деплоя
pub struct DeploymentExecutionParams<'a> {
    pub config: &'a Config,
    pub deployment_name: &'a str,
    #[allow(dead_code)] // Используется внутри других методов
    pub history_path: &'a str,
    pub failed_deployments: &'a Arc<Mutex<Vec<String>>>,
}

/// Структура для параметров запуска всех деплоев
pub struct AllDeploymentsParams<'a> {
    pub event: Option<&'a str>,
    #[allow(dead_code)] // Используется внутри метода run_all_deployments
    pub parallel: bool,
    pub failed_deployments: Arc<Mutex<Vec<String>>>,
}

/// Структура, отвечающая за работу с деплоем
pub struct Deployment {
    config: Config,
    history_path: String,
    parallel_mode: bool,
}

impl Deployment {
    /// Создаёт новый экземпляр Deployment
    ///
    /// # Параметры
    ///
    /// * `config` - Конфигурация деплоя
    /// * `history_path` - Путь к файлу истории деплоев
    /// * `parallel_mode` - Режим выполнения (параллельный или последовательный)
    pub fn new(config: Config, history_path: String, parallel_mode: bool) -> Self {
        Self {
            config,
            history_path,
            parallel_mode,
        }
    }

    /// Выполняет указанное событие для деплоя
    ///
    /// # Параметры
    ///
    /// * `deployment` - Имя деплоя
    /// * `event` - Имя события
    pub async fn run_specific_event(&self, deployment: &str, event: &str) {
        info!(
            "Запуск команд для деплоя '{}', событие: '{}'",
            deployment, event
        );

        // Создаем менеджер деплоев
        let deployment_manager = super::deployment_manager::DeploymentManager::new(
            &self.config,
            &self.history_path,
            false,
        );

        // Выполняем указанное событие деплоя
        if !deployment_manager.execute_event(deployment, event).await {
            error!("Ошибка выполнения команд для деплоя '{}'", deployment);
            exit(1);
        }

        info!("Все команды выполнены успешно");
    }

    /// Запускает все события для указанного деплоя последовательно
    ///
    /// # Параметры
    ///
    /// * `deployment` - Имя деплоя
    pub async fn run_all_events(&self, deployment: &str) {
        info!("Запуск всех событий для деплоя '{}'", deployment);

        // Создаем менеджер деплоев
        let deployment_manager = super::deployment_manager::DeploymentManager::new(
            &self.config,
            &self.history_path,
            false,
        );

        // Выполняем все события деплоя
        if !deployment_manager.execute_all_events(deployment).await {
            error!("Ошибка выполнения событий для деплоя '{}'", deployment);
            exit(1);
        }
    }

    /// Выводит список всех доступных деплоев и команд
    pub fn list_deployments(&self) {
        // Создаем менеджер деплоев с любыми параметрами, т.к. они не используются при выводе списка
        let deployment_manager = super::deployment_manager::DeploymentManager::new(
            &self.config,
            &self.history_path,
            false,
        );
        deployment_manager.list_deployments();
    }

    /// Проверяет конфигурацию деплоя на корректность
    ///
    /// # Параметры
    ///
    /// * `deployment_name` - Имя деплоя для проверки
    pub fn verify_deployment_config(&self, deployment_name: &str) {
        info!("Проверка конфигурации деплоя: {}", deployment_name);
        match crate::config::verify_deployment(&self.config, deployment_name) {
            Ok(true) => info!("Конфигурация деплоя '{}' корректна", deployment_name),
            Ok(false) => {
                error!("Конфигурация деплоя '{}' некорректна", deployment_name);
                exit(1);
            }
            Err(e) => {
                error!("Ошибка проверки конфигурации: {}", e);
                exit(1);
            }
        }
    }

    /// Показывает историю деплоев
    ///
    /// # Параметры
    ///
    /// * `deployment_name` - Имя деплоя
    /// * `limit` - Количество записей истории для отображения
    pub fn show_deployment_history(&self, deployment_name: &str, limit: usize) {
        match history::display_deployment_history(&self.history_path, deployment_name, limit) {
            Ok(_) => {}
            Err(e) => {
                error!("Ошибка отображения истории деплоя: {}", e);
                exit(1);
            }
        }
    }

    /// Очищает историю деплоев
    ///
    /// # Параметры
    ///
    /// * `deployment_name` - Имя деплоя (если None, очищается вся история)
    pub fn clear_deployment_history(&self, deployment_name: Option<&str>) {
        match history::clear_history(&self.history_path, deployment_name) {
            Ok(_) => {}
            Err(e) => {
                error!("Ошибка очистки истории: {}", e);
                exit(1);
            }
        }
    }

    /// Записывает событие о начале полного деплоя
    fn record_full_deploy_start(&self) {
        if let Err(e) = storage::record_deployment(
            &self.history_path,
            "all-deployments",
            "start-full-deploy-all",
            true,
            None,
        ) {
            warn!("Ошибка записи события: {}", e);
        }
    }

    /// Выполняет конкретное событие для указанного деплоя
    ///
    /// # Параметры
    ///
    /// * `deployment_name` - Имя деплоя
    /// * `event_name` - Имя события
    /// * `failed_deployments` - Список неудачных деплоев (мутируется в процессе выполнения)
    ///
    /// # Возвращаемое значение
    ///
    /// Результат выполнения деплоя (true - успешно, false - с ошибками)
    async fn execute_single_event(
        &self,
        deployment_name: &str,
        event_name: &str,
        failed_deployments: &Arc<Mutex<Vec<String>>>,
    ) -> bool {
        let params = EventExecutionParams {
            config: &self.config,
            deployment_name,
            event_name,
            history_path: &self.history_path,
        };

        self.execute_event_with_params(&params, failed_deployments)
            .await
    }

    /// Выполняет указанное событие с заданными параметрами
    ///
    /// # Параметры
    ///
    /// * `params` - Параметры выполнения события
    /// * `failed_deployments` - Список неудачных деплоев
    ///
    /// # Возвращаемое значение
    ///
    /// Результат выполнения (true - успешно, false - с ошибками)
    #[allow(unused_variables)]
    async fn execute_event_with_params(
        &self,
        params: &EventExecutionParams<'_>,
        failed_deployments: &Arc<Mutex<Vec<String>>>,
    ) -> bool {
        match command_runner::execute_command(
            params.config,
            params.deployment_name,
            params.event_name,
            params.history_path,
        )
        .await
        {
            Ok(_) => {
                info!(
                    "Деплой '{}', событие '{}' успешно выполнено",
                    params.deployment_name, params.event_name
                );
                true
            }
            Err(e) => {
                error!(
                    "Ошибка выполнения деплоя '{}', событие '{}': {}",
                    params.deployment_name, params.event_name, e
                );
                let mut failed = failed_deployments.lock().unwrap();
                failed.push(params.deployment_name.to_string());
                false
            }
        }
    }

    /// Записывает результат выполнения всех деплоев
    ///
    /// # Параметры
    ///
    /// * `all_success` - Флаг успешности всех деплоев
    /// * `failed_list` - Список неудачных деплоев
    /// * `is_parallel` - Режим выполнения (параллельный или последовательный)
    fn record_deploy_all_result(
        &self,
        all_success: bool,
        failed_list: &[String],
        is_parallel: bool,
    ) {
        let mode_str = if is_parallel {
            "параллельный"
        } else {
            "последовательный"
        };

        if all_success {
            info!("Все деплои успешно выполнены ({} режим)", mode_str);
            if let Err(e) = command_runner::record_success(
                &self.history_path,
                "all-deployments",
                "complete-full-deploy-all",
                Some("Все деплои успешно завершены".to_string()),
            ) {
                warn!("Ошибка записи события: {}", e);
            }
        } else {
            error!(
                "Некоторые деплои завершились с ошибками ({} режим): {}",
                mode_str,
                failed_list.join(", ")
            );
            if let Err(e) = command_runner::record_failure(
                &self.history_path,
                "all-deployments",
                "failed-full-deploy-all",
                format!("Деплои с ошибками: {}", failed_list.join(", ")),
            ) {
                warn!("Ошибка записи события: {}", e);
            }
            exit(1);
        }
    }

    /// Запускает процесс записи событий для деплоя
    ///
    /// # Параметры
    ///
    /// * `deployment_name` - Имя деплоя
    ///
    /// # Возвращаемое значение
    ///
    /// Успешность операции записи события
    fn record_deployment_start(&self, deployment_name: &str) -> bool {
        match storage::record_deployment(
            &self.history_path,
            deployment_name,
            "start-full-deploy",
            true,
            None,
        ) {
            Ok(_) => true,
            Err(e) => {
                warn!("Ошибка записи события: {}", e);
                false
            }
        }
    }

    /// Обрабатывает ошибку выполнения события
    ///
    /// # Параметры
    ///
    /// * `deployment_name` - Имя деплоя
    /// * `event_name` - Имя события
    /// * `error` - Описание ошибки
    ///
    /// # Возвращаемое значение
    ///
    /// Результат записи ошибки
    fn handle_event_error(&self, deployment_name: &str, event_name: &str, error: String) -> bool {
        error!("Ошибка выполнения события '{}': {}", event_name, error);
        if let Err(log_err) = command_runner::record_failure(
            &self.history_path,
            deployment_name,
            &format!("failed-{}", event_name),
            error,
        ) {
            warn!("Ошибка записи события: {}", log_err);
        }
        false
    }

    /// Записывает результат успешного выполнения всех событий деплоя
    ///
    /// # Параметры
    ///
    /// * `deployment_name` - Имя деплоя
    fn record_deployment_success(&self, deployment_name: &str) {
        info!(
            "Все события для деплоя '{}' успешно выполнены",
            deployment_name
        );
        if let Err(e) = command_runner::record_success(
            &self.history_path,
            deployment_name,
            "complete-full-deploy",
            Some("Все события деплоя успешно завершены".to_string()),
        ) {
            warn!("Ошибка записи события: {}", e);
        }
    }

    /// Записывает результат ошибки выполнения деплоя
    ///
    /// # Параметры
    ///
    /// * `deployment_name` - Имя деплоя
    /// * `failed_deployments` - Список неудачных деплоев
    fn record_deployment_failure(
        &self,
        deployment_name: &str,
        failed_deployments: &Arc<Mutex<Vec<String>>>,
    ) {
        error!("Деплой '{}' завершился с ошибками", deployment_name);
        if let Err(e) = command_runner::record_failure(
            &self.history_path,
            deployment_name,
            "failed-full-deploy",
            "Одно из событий завершилось с ошибкой".to_string(),
        ) {
            warn!("Ошибка записи события: {}", e);
        }

        // Добавляем имя деплоя в список неудачных
        let mut failed = failed_deployments.lock().unwrap();
        failed.push(deployment_name.to_string());
    }

    /// Выполняет события для деплоя
    ///
    /// # Параметры
    ///
    /// * `deployment_name` - Имя деплоя
    /// * `events` - Список событий для выполнения
    /// * `failed_deployments` - Список неудачных деплоев
    ///
    /// # Возвращаемое значение
    ///
    /// Успешность выполнения всех событий
    #[allow(unused_variables)]
    async fn execute_events_sequence(
        &self,
        deployment_name: &str,
        events: &[crate::config::Event],
        failed_deployments: &Arc<Mutex<Vec<String>>>,
    ) -> bool {
        let mut deployment_success = true;

        // Выполняем все события последовательно
        for event in events {
            info!(
                "Запуск события '{}' для деплоя '{}'",
                event.name, deployment_name
            );

            // Создаем параметры для выполнения события
            let params = EventExecutionParams {
                config: &self.config,
                deployment_name,
                event_name: &event.name,
                history_path: &self.history_path,
            };

            match command_runner::execute_command(
                params.config,
                params.deployment_name,
                params.event_name,
                params.history_path,
            )
            .await
            {
                Ok(_) => {
                    info!("Событие '{}' успешно выполнено", event.name);
                }
                Err(e) => {
                    deployment_success =
                        self.handle_event_error(deployment_name, &event.name, e.to_string());
                    break;
                }
            }
        }

        deployment_success
    }

    /// Выполняет все события для указанного деплоя последовательно
    ///
    /// # Параметры
    ///
    /// * `deployment_name` - Имя деплоя
    /// * `failed_deployments` - Список неудачных деплоев
    ///
    /// # Возвращаемое значение
    ///
    /// Результат выполнения всех событий (true - успешно, false - с ошибками)
    async fn execute_all_events_for_deployment(
        &self,
        deployment_name: &str,
        failed_deployments: &Arc<Mutex<Vec<String>>>,
    ) -> bool {
        let params = DeploymentExecutionParams {
            config: &self.config,
            deployment_name,
            history_path: &self.history_path,
            failed_deployments,
        };

        self.process_deployment_events(&params).await
    }

    /// Выполняет обработку всех событий деплоя
    ///
    /// # Параметры
    ///
    /// * `params` - Параметры выполнения деплоя
    ///
    /// # Возвращаемое значение
    ///
    /// Результат выполнения всех событий (true - успешно, false - с ошибками)
    async fn process_deployment_events(&self, params: &DeploymentExecutionParams<'_>) -> bool {
        // history_path используется здесь через record_deployment_start и другие методы
        match deployments::get_deployment_config(params.config, params.deployment_name) {
            Ok(dep_config) => {
                // Запись события начала деплоя
                self.record_deployment_start(params.deployment_name);

                // Выполняем все события последовательно
                let deployment_success = self
                    .execute_events_sequence(
                        params.deployment_name,
                        &dep_config.events,
                        params.failed_deployments,
                    )
                    .await;

                // Запись результата деплоя
                if deployment_success {
                    self.record_deployment_success(params.deployment_name);
                } else {
                    self.record_deployment_failure(
                        params.deployment_name,
                        params.failed_deployments,
                    );
                }

                deployment_success
            }
            Err(e) => {
                error!(
                    "Ошибка получения конфигурации деплоя '{}': {}",
                    params.deployment_name, e
                );
                let mut failed = params.failed_deployments.lock().unwrap();
                failed.push(params.deployment_name.to_string());
                false
            }
        }
    }

    /// Запускает все деплои последовательно
    ///
    /// # Параметры
    ///
    /// * `params` - Параметры запуска всех деплоев
    ///
    /// # Возвращаемое значение
    ///
    /// true если все деплои выполнены успешно, false если были ошибки
    async fn run_deployments_sequentially(&self, params: &AllDeploymentsParams<'_>) -> bool {
        let mut all_success = true;

        for deployment in &self.config.deployments {
            let deployment_name = &deployment.name;
            info!(
                "Запуск деплоя '{}' (последовательный режим)",
                deployment_name
            );

            // Обрабатываем деплой в зависимости от наличия события
            let success = if let Some(event_name) = params.event {
                // Запускаем конкретное событие
                self.execute_single_event(deployment_name, event_name, &params.failed_deployments)
                    .await
            } else {
                // Запускаем все события для деплоя
                self.execute_all_events_for_deployment(deployment_name, &params.failed_deployments)
                    .await
            };

            if !success {
                all_success = false;
            }
        }

        all_success
    }

    /// Создает и запускает задачу для параллельного выполнения деплоя
    ///
    /// # Параметры
    ///
    /// * `deployment_name` - Имя деплоя
    /// * `event` - Опциональное имя события
    /// * `failed_deployments` - Список неудачных деплоев
    ///
    /// # Возвращаемое значение
    ///
    /// Кортеж (имя деплоя, результат выполнения)
    async fn run_deployment_task(
        &self,
        deployment_name: String,
        event: Option<String>,
        failed_deployments: Arc<Mutex<Vec<String>>>,
    ) -> (String, bool) {
        info!("Запуск деплоя '{}' (параллельный режим)", deployment_name);

        // Результат выполнения деплоя
        let success = if let Some(event_name) = event {
            // Создаем параметры для выполнения события
            let event_params = EventExecutionParams {
                config: &self.config,
                deployment_name: &deployment_name,
                event_name: &event_name,
                history_path: &self.history_path,
            };

            // Выполняем событие
            self.execute_event_with_params(&event_params, &failed_deployments)
                .await
        } else {
            // Создаем параметры для выполнения всех событий деплоя
            let deployment_params = DeploymentExecutionParams {
                config: &self.config,
                deployment_name: &deployment_name,
                history_path: &self.history_path,
                failed_deployments: &failed_deployments,
            };

            // Выполняем все события для деплоя
            self.process_deployment_events(&deployment_params).await
        };

        // Возвращаем результат выполнения деплоя
        (deployment_name, success)
    }

    /// Запускает все деплои параллельно
    ///
    /// # Параметры
    ///
    /// * `params` - Параметры запуска всех деплоев
    ///
    /// # Возвращаемое значение
    ///
    /// true если все деплои выполнены успешно, false если были ошибки
    async fn run_deployments_in_parallel(&self, params: &AllDeploymentsParams<'_>) -> bool {
        use tokio::task::JoinSet;

        // Создаем набор задач для параллельного выполнения
        info!("Запуск деплоев в параллельном режиме");
        let mut tasks = JoinSet::new();

        // Добавляем все деплои в JoinSet для параллельного выполнения
        for deployment in &self.config.deployments {
            let deployment_name = deployment.name.clone();
            let event_clone = params.event.map(|e| e.to_string());
            let failed_deployments_clone = Arc::clone(&params.failed_deployments);

            // Клонируем self для передачи в задачу
            let deployment_self = self.clone();

            // Запускаем отдельную задачу для каждого деплоя
            tasks.spawn(async move {
                deployment_self
                    .run_deployment_task(deployment_name, event_clone, failed_deployments_clone)
                    .await
            });
        }

        // Дожидаемся завершения всех задач
        let mut all_success = true;
        while let Some(result) = tasks.join_next().await {
            if let Ok((name, success)) = result {
                if !success {
                    all_success = false;
                    info!("Деплой '{}' завершился с ошибками", name);
                }
            } else {
                error!("Ошибка выполнения задачи деплоя");
                all_success = false;
            }
        }

        all_success
    }

    /// Запускает все доступные деплои в конфигурации
    ///
    /// # Параметры
    ///
    /// * `event` - Опциональное имя события для запуска (если None - запускаются все события)
    /// * `parallel` - Флаг, указывающий на режим выполнения (true - параллельный, false - последовательный)
    pub async fn run_all_deployments(&self, event: Option<&str>, parallel: bool) {
        info!(
            "Запуск всех доступных деплоев в режиме {}",
            if parallel {
                "параллельном"
            } else {
                "последовательном"
            }
        );

        if self.config.deployments.is_empty() {
            warn!("В конфигурации не найдено ни одного деплоя");
            return;
        }

        info!(
            "Найдено {} деплоев для выполнения",
            self.config.deployments.len()
        );

        // Запись события начала полного деплоя
        self.record_full_deploy_start();

        // Используем Arc и Mutex для общего доступа к результатам из разных тасков
        let failed_deployments = Arc::new(Mutex::new(Vec::new()));

        // Создаем параметры для запуска деплоев
        let deploy_params = AllDeploymentsParams {
            event,
            parallel, // Используется для выбора режима запуска деплоев
            failed_deployments: Arc::clone(&failed_deployments),
        };

        // Запускаем деплои в параллельном или последовательном режиме
        let all_success = if parallel {
            self.run_deployments_in_parallel(&deploy_params).await
        } else {
            self.run_deployments_sequentially(&deploy_params).await
        };

        // Получаем список неудачных деплоев
        let failed_list = failed_deployments.lock().unwrap().clone();

        // Запись итогового результата
        self.record_deploy_all_result(all_success, &failed_list, parallel);
    }
}

impl Clone for Deployment {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            history_path: self.history_path.clone(),
            parallel_mode: self.parallel_mode,
        }
    }
}

/// Создает шаблон деплоя с заданным именем
///
/// # Параметры
///
/// * `deployment_name` - Имя нового деплоя
/// * `config_path` - Путь к файлу конфигурации
pub fn create_deployment_template(deployment_name: &str, config_path: &str) {
    info!("Создание шаблона деплоя: {}", deployment_name);
    match crate::config::create_template_deployment(deployment_name, config_path) {
        Ok(_) => info!("Шаблон деплоя '{}' успешно создан", deployment_name),
        Err(e) => {
            error!("Ошибка создания шаблона: {}", e);
            exit(1);
        }
    }
}
