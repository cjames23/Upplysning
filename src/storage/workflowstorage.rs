use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::model::stepresult::StepResult;
use crate::model::workflow::Workflow;
use crate::model::workflowinstance::WorkflowInstance;

#[async_trait]
pub trait WorkflowStorage: Send + Sync {
    async fn save_workflow(&self, workflow: &Workflow) -> Result<(), String>;
    async fn get_workflow(&self, id: Uuid) -> Result<Workflow, String>;
    async fn save_instance(&self, instance: &WorkflowInstance) -> Result<(), String>;
    async fn get_instance(&self, id: Uuid) -> Result<WorkflowInstance, String>;
    async fn update_step_result(&self, instance_id: Uuid, step_id: Uuid, result: &StepResult) -> Result<(), String>;
}

pub struct InMemoryStorage {
    workflows: Arc<RwLock<HashMap<Uuid, Workflow>>>,
    instances: Arc<RwLock<HashMap<Uuid, WorkflowInstance>>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            workflows: Arc::new(RwLock::new(HashMap::new())),
            instances: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl WorkflowStorage for InMemoryStorage {
    async fn save_workflow(&self, workflow: &Workflow) -> Result<(), String> {
        let mut workflows = self.workflows.write().await;
        workflows.insert(workflow.id, workflow.clone());
        Ok(())
    }

    async fn get_workflow(&self, id: Uuid) -> Result<Workflow, String> {
        let workflows = self.workflows.read().await;
        workflows.get(&id)
            .cloned()
            .ok_or_else(|| format!("Workflow not found: {}", id))
    }

    async fn save_instance(&self, instance: &WorkflowInstance) -> Result<(), String> {
        let mut instances = self.instances.write().await;
        instances.insert(instance.id, instance.clone());
        Ok(())
    }

    async fn get_instance(&self, id: Uuid) -> Result<WorkflowInstance, String> {
        let instances = self.instances.read().await;
        instances.get(&id)
            .cloned()
            .ok_or_else(|| format!("Instance not found: {}", id))
    }

    async fn update_step_result(&self, instance_id: Uuid, step_id: Uuid, result: &StepResult) -> Result<(), String> {
        let mut instances = self.instances.write().await;
        let instance = instances.get_mut(&instance_id)
            .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

        instance.step_results.insert(step_id, result.clone());
        instance.updated_at = chrono::Utc::now();

        Ok(())
    }
}