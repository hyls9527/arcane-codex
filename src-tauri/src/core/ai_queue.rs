use crate::core::db::Database;
use crate::core::lm_studio::{LMStudioClient, LMStudioConfig, AIResult};
use crate::utils::error::AppResult;
use async_channel::{bounded, Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, Semaphore};
use tracing::{info, warn, error, debug};

const QUEUE_CAPACITY: usize = 1000;
const DEFAULT_CONCURRENCY: usize = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStatus {
    pub is_running: bool,
    pub is_paused: bool,
    pub total_tasks: usize,
    pub processed_tasks: usize,
    pub failed_tasks: usize,
    pub pending_tasks: usize,
    pub concurrency: usize,
}

#[derive(Debug, Clone)]
pub struct AITask {
    pub image_id: i64,
    pub file_path: String,
    pub retry_count: i32,
}

#[derive(Debug, Clone)]
pub enum QueueCommand {
    Pause,
    Resume,
    Cancel,
}

pub struct AITaskQueue {
    sender: Sender<AITask>,
    receiver: Receiver<AITask>,
    command_sender: broadcast::Sender<QueueCommand>,
    semaphore: Arc<Semaphore>,
    is_running: Arc<AtomicBool>,
    is_paused: Arc<AtomicBool>,
    total_tasks: Arc<AtomicUsize>,
    processed_tasks: Arc<AtomicUsize>,
    failed_tasks: Arc<AtomicUsize>,
    db: Arc<Database>,
    client_config: LMStudioConfig,
}

impl AITaskQueue {
    pub fn new(db: Arc<Database>, concurrency: Option<usize>) -> Self {
        let conc = concurrency.unwrap_or(DEFAULT_CONCURRENCY);
        let (sender, receiver) = bounded::<AITask>(QUEUE_CAPACITY);
        let (command_sender, _) = broadcast::channel::<QueueCommand>(16);

        info!(
            "AI 任务队列初始化: 容量 {}, 并发数 {}",
            QUEUE_CAPACITY, conc
        );

        Self {
            sender,
            receiver,
            command_sender,
            semaphore: Arc::new(Semaphore::new(conc)),
            is_running: Arc::new(AtomicBool::new(false)),
            is_paused: Arc::new(AtomicBool::new(false)),
            total_tasks: Arc::new(AtomicUsize::new(0)),
            processed_tasks: Arc::new(AtomicUsize::new(0)),
            failed_tasks: Arc::new(AtomicUsize::new(0)),
            db,
            client_config: LMStudioConfig::default(),
        }
    }

    pub fn get_status(&self) -> QueueStatus {
        let total = self.total_tasks.load(Ordering::SeqCst);
        let processed = self.processed_tasks.load(Ordering::SeqCst);
        let failed = self.failed_tasks.load(Ordering::SeqCst);
        let pending = self.sender.len();

        QueueStatus {
            is_running: self.is_running.load(Ordering::SeqCst),
            is_paused: self.is_paused.load(Ordering::SeqCst),
            total_tasks: total,
            processed_tasks: processed,
            failed_tasks: failed,
            pending_tasks: pending,
            concurrency: self.semaphore.available_permits(),
        }
    }

    pub async fn enqueue(&self, task: AITask) -> AppResult<()> {
        if !self.is_running.load(Ordering::SeqCst) {
            return Err(crate::utils::error::AppError::Validation(
                "队列未运行，无法添加任务".to_string(),
            ));
        }

        if self.sender.len() >= QUEUE_CAPACITY {
            return Err(crate::utils::error::AppError::Validation(
                "队列已满，请稍后重试".to_string(),
            ));
        }

        self.sender.send(task).await.map_err(|e| {
            crate::utils::error::AppError::Validation(format!("添加任务失败: {}", e))
        })?;

        self.total_tasks.fetch_add(1, Ordering::SeqCst);
        debug!("任务入队成功，队列长度: {}", self.sender.len());

        Ok(())
    }

    pub fn start(&mut self) {
        self.is_running.store(true, Ordering::SeqCst);
        self.is_paused.store(false, Ordering::SeqCst);
        info!("AI 任务队列已启动");
    }

    pub fn pause(&self) {
        self.is_paused.store(true, Ordering::SeqCst);
        info!("AI 任务队列已暂停");
    }

    pub fn resume(&self) {
        self.is_paused.store(false, Ordering::SeqCst);
        info!("AI 任务队列已恢复");
    }

    pub fn cancel(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        info!("AI 任务队列已取消");
    }

    pub fn get_command_receiver(&self) -> broadcast::Receiver<QueueCommand> {
        self.command_sender.subscribe()
    }

    pub async fn start_workers(&self, worker_count: usize) {
        let mut handles = vec![];

        for i in 0..worker_count {
            let receiver = self.receiver.clone();
            let semaphore = self.semaphore.clone();
            let is_paused = self.is_paused.clone();
            let is_running = self.is_running.clone();
            let processed = self.processed_tasks.clone();
            let failed = self.failed_tasks.clone();
            let db = self.db.clone();
            let client_config = self.client_config.clone();
            let mut cmd_receiver = self.get_command_receiver();

            let handle = tokio::spawn(async move {
                info!("Worker {} 启动", i);

                loop {
                    if !is_running.load(Ordering::SeqCst) {
                        info!("Worker {} 检测到停止信号", i);
                        break;
                    }

                    if is_paused.load(Ordering::SeqCst) {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        continue;
                    }

                    tokio::select! {
                        Ok(cmd) = cmd_receiver.recv() => {
                            match cmd {
                                QueueCommand::Cancel => {
                                    info!("Worker {} 收到取消命令", i);
                                    break;
                                }
                                QueueCommand::Pause => {
                                    is_paused.store(true, Ordering::SeqCst);
                                }
                                QueueCommand::Resume => {
                                    is_paused.store(false, Ordering::SeqCst);
                                }
                            }
                        }
                        task_result = receiver.recv() => {
                            match task_result {
                                Ok(task) => {
                                    let permit = semaphore.clone().acquire_owned().await;
                                    if let Ok(_permit) = permit {
                                        let result = Self::process_task(&task, &db, &client_config).await;
                                        match result {
                                            Ok(_) => {
                                                processed.fetch_add(1, Ordering::SeqCst);
                                                debug!("Worker {}: 任务 {} 处理成功", i, task.image_id);
                                            }
                                            Err(e) => {
                                                failed.fetch_add(1, Ordering::SeqCst);
                                                error!("Worker {}: 任务 {} 处理失败: {}", i, task.image_id, e);
                                            }
                                        }
                                    }
                                }
                                Err(_) => {
                                    debug!("Worker {} 队列为空", i);
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                                }
                            }
                        }
                    }
                }

                info!("Worker {} 退出", i);
            });

            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.await;
        }

        info!("所有 Worker 已退出");
    }

    async fn process_task(
        task: &AITask,
        db: &Database,
        config: &LMStudioConfig,
    ) -> AppResult<()> {
        let client = LMStudioClient::new(config.clone())?;

        let result = client.analyze_image(&task.file_path).await?;

        Self::update_image_ai_result(db, task.image_id, &result).await?;

        info!(
            "任务 {} 处理完成: {} (置信度: {:.2})",
            task.image_id, task.file_path, result.confidence
        );

        Ok(())
    }

    async fn update_image_ai_result(
        db: &Database,
        image_id: i64,
        result: &AIResult,
    ) -> AppResult<()> {
        let conn = db.open_connection()?;

        let tags_json = serde_json::to_string(&result.tags).unwrap_or("[]".to_string());

        conn.execute(
            "UPDATE images SET 
             ai_status = 'completed',
             ai_tags = ?1,
             ai_description = ?2,
             ai_category = ?3,
             ai_confidence = ?4,
             ai_model = ?5,
             ai_processed_at = datetime('now'),
             ai_error_message = NULL,
             updated_at = datetime('now')
             WHERE id = ?6",
            rusqlite::params![
                tags_json,
                result.description,
                result.category,
                result.confidence,
                result.raw_response.chars().take(500).collect::<String>(),
                image_id,
            ],
        )?;

        debug!("更新图片 {} 的 AI 结果", image_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn setup_test_db() -> (Arc<Database>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_queue.db");
        let db = Arc::new(Database::new_from_path(db_path.to_str().unwrap()).unwrap());
        db.init().unwrap();
        (db, temp_dir)
    }

    #[test]
    fn test_queue_creation() {
        let (db, _temp) = setup_test_db();
        let queue = AITaskQueue::new(db, None);

        let status = queue.get_status();
        assert!(!status.is_running);
        assert!(!status.is_paused);
        assert_eq!(status.total_tasks, 0);
        assert_eq!(status.processed_tasks, 0);
        assert_eq!(status.failed_tasks, 0);
        assert_eq!(status.pending_tasks, 0);
        assert_eq!(status.concurrency, DEFAULT_CONCURRENCY);
    }

    #[test]
    fn test_queue_custom_concurrency() {
        let (db, _temp) = setup_test_db();
        let queue = AITaskQueue::new(db, Some(5));

        let status = queue.get_status();
        assert_eq!(status.concurrency, 5);
    }

    #[test]
    fn test_queue_start_stop() {
        let (db, _temp) = setup_test_db();
        let mut queue = AITaskQueue::new(db, None);

        assert!(!queue.get_status().is_running);

        queue.start();
        assert!(queue.get_status().is_running);
        assert!(!queue.get_status().is_paused);

        queue.pause();
        assert!(queue.get_status().is_running);
        assert!(queue.get_status().is_paused);

        queue.resume();
        assert!(queue.get_status().is_running);
        assert!(!queue.get_status().is_paused);

        queue.cancel();
        assert!(!queue.get_status().is_running);
    }

    #[test]
    fn test_enqueue_when_not_running() {
        let (db, _temp) = setup_test_db();
        let queue = AITaskQueue::new(db, None);

        let task = AITask {
            image_id: 1,
            file_path: "test.jpg".to_string(),
            retry_count: 0,
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(queue.enqueue(task));
        assert!(result.is_err());
    }

    #[test]
    fn test_enqueue_success() {
        let (db, _temp) = setup_test_db();
        let mut queue = AITaskQueue::new(db, None);
        queue.start();

        let task = AITask {
            image_id: 1,
            file_path: "test.jpg".to_string(),
            retry_count: 0,
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(queue.enqueue(task));
        assert!(result.is_ok());

        let status = queue.get_status();
        assert_eq!(status.total_tasks, 1);
        assert_eq!(status.pending_tasks, 1);
    }

    #[test]
    fn test_queue_capacity_limit() {
        let (db, _temp) = setup_test_db();
        let mut queue = AITaskQueue::new(db, Some(1));
        queue.start();

        let rt = tokio::runtime::Runtime::new().unwrap();

        for i in 0..QUEUE_CAPACITY {
            let task = AITask {
                image_id: i as i64,
                file_path: format!("test_{}.jpg", i),
                retry_count: 0,
            };

            let result = rt.block_on(queue.enqueue(task));
            if i >= QUEUE_CAPACITY - 1 {
                assert!(result.is_err() || queue.sender.is_full());
            }
        }
    }

    #[tokio::test]
    async fn test_command_broadcast() {
        let (db, _temp) = setup_test_db();
        let queue = AITaskQueue::new(db, None);

        let mut receiver = queue.get_command_receiver();

        queue
            .command_sender
            .send(QueueCommand::Pause)
            .unwrap();

        let cmd = receiver.recv().await.unwrap();
        match cmd {
            QueueCommand::Pause => {}
            _ => panic!("Expected Pause command"),
        }
    }

    #[test]
    fn test_atomic_counters() {
        let (db, _temp) = setup_test_db();
        let queue = AITaskQueue::new(db, None);

        queue.total_tasks.store(10, Ordering::SeqCst);
        queue.processed_tasks.store(7, Ordering::SeqCst);
        queue.failed_tasks.store(2, Ordering::SeqCst);

        let status = queue.get_status();
        assert_eq!(status.total_tasks, 10);
        assert_eq!(status.processed_tasks, 7);
        assert_eq!(status.failed_tasks, 2);
        assert_eq!(status.pending_tasks, 0);
    }
}
