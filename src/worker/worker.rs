use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use crate::model::stepresult::StepResult;
use crate::model::stepstatus::StepStatus;
use crate::model::tasktype::TaskType;
pub(crate) use crate::model::workertask::WorkerTask;
use crate::model::worker::Worker;
use crate::scheduler::scheduler::Scheduler;
use crate::storage::workflowstorage::WorkflowStorage;

impl Worker {
    pub fn new(
        id: String,
        capabilities: Vec<TaskType>,
        scheduler: Arc<Scheduler>,
        storage: Arc<dyn WorkflowStorage>,
        task_receiver: mpsc::Receiver<WorkerTask>,
    ) -> Self {
        Self {
            id,
            capabilities,
            scheduler,
            storage,
            task_receiver,
        }
    }

    pub async fn start(&mut self) -> Result<(), String> {
        // Register with scheduler
        self.scheduler.register_worker(self.id.clone(), self.capabilities.clone()).await?;

        // Start heartbeat
        let scheduler = self.scheduler.clone();
        let worker_id = self.id.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            loop {
                interval.tick().await;
                if let Err(e) = scheduler.worker_heartbeat(&worker_id).await {
                    eprintln!("Heartbeat failed: {}", e);
                }
            }
        });

        // Process tasks
        while let Some(task) = self.task_receiver.recv().await {
            self.process_task(task).await?;
        }

        Ok(())
    }

    async fn process_task(&self, task: WorkerTask) -> Result<(), String> {
        // Update step status to Running
        let mut result = StepResult {
            step_id: task.step_id,
            status: StepStatus::Running,
            output: None,
            error: None,
            worker_id: Some(self.id.clone()),
            started_at: Some(chrono::Utc::now()),
            completed_at: None,
        };

        self.storage.update_step_result(task.instance_id, task.step_id, &result).await
            .map_err(|e| e.to_string())?;

        // Execute the task based on type
        let task_result = match task.task_type {
            TaskType::DataPreparation => self.execute_data_preparation(&task).await,
            TaskType::ModelTraining => self.execute_model_training(&task).await,
            TaskType::ModelEvaluation => self.execute_model_evaluation(&task).await,
            TaskType::ModelDeployment => self.execute_model_deployment(&task).await,
            TaskType::Notification => self.execute_notification(&task).await,
            TaskType::Custom(ref name) => self.execute_custom_task(&task, name).await,
        };

        // Update step result based on execution outcome
        match task_result {
            Ok(output) => {
                result.status = StepStatus::Completed;
                result.output = Some(output);
            },
            Err(e) => {
                result.status = StepStatus::Failed;
                result.error = Some(e);
            }
        }

        result.completed_at = Some(chrono::Utc::now());

        // Update step result
        self.storage.update_step_result(task.instance_id, task.step_id, &result).await
            .map_err(|e| e.to_string())?;

        // Check for feedback loops
        self.scheduler.clone().process_feedback(task.instance_id, task.step_id).await?;

        Ok(())
    }

    async fn execute_data_preparation(&self, task: &WorkerTask) -> Result<serde_json::Value, String> {
        // Implementation for data preparation tasks
        Ok(serde_json::json!({"status": "completed"}))
    }

    async fn execute_model_training(&self, task: &WorkerTask) -> Result<serde_json::Value, String> {
        // Implementation for model training tasks
        Ok(serde_json::json!({"status": "completed", "metrics": {"accuracy": 0.95}}))
    }

    async fn execute_model_evaluation(&self, task: &WorkerTask) -> Result<serde_json::Value, String> {
        // Implementation for model evaluation tasks
        Ok(serde_json::json!({"status": "completed", "metrics": {"precision": 0.92, "recall": 0.89}}))
    }

    async fn execute_model_deployment(&self, task: &WorkerTask) -> Result<serde_json::Value, String> {
        // Implementation for model deployment tasks
        Ok(serde_json::json!({"status": "completed", "endpoint": "https://api.example.com/models/123"}))
    }

    async fn execute_notification(&self, task: &WorkerTask) -> Result<serde_json::Value, String> {
        // Implementation for notification tasks
        Ok(serde_json::json!({"status": "completed"}))
    }

    async fn execute_custom_task(&self, task: &WorkerTask, name: &str) -> Result<serde_json::Value, String> {
        // Implementation for custom tasks
        Ok(serde_json::json!({"status": "completed", "task": name}))
    }
}