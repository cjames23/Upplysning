use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use std::time::Duration;
use tokio::sync::{RwLock, mpsc};
use crate::model::tasktype::TaskType;
use crate::storage::workflowstorage::WorkflowStorage;
use crate::model::workflow::Workflow;
use crate::model::workflowinstance::WorkflowInstance;
use crate::model::step::Step;
use crate::model::workertask::WorkerTask;
use crate::model::feedbackloop::FeedbackLoop;
use crate::worker::worker;

pub struct Scheduler {
    workers: Arc<RwLock<HashMap<String, WorkerInfo>>>,
    task_queue: Arc<RwLock<Option<lapin::Channel>>>,
    consensus: Arc<RwLock<SimpleConsensus>>,
    storage: Arc<dyn WorkflowStorage>,
}

struct WorkerInfo {
    id: String,
    last_heartbeat: chrono::DateTime<chrono::Utc>,
    capabilities: Vec<TaskType>,
}

// Simplified consensus mechanism since paxos-rust doesn't provide the needed API
struct SimpleConsensus {
    node_id: usize,
    nodes: Vec<String>,
    is_leader: bool,
    term: u64,
}

impl SimpleConsensus {
    pub fn new(node_id: usize, nodes: Vec<String>) -> Self {
        // Simple implementation - first node is always leader
        let is_leader = node_id == 1;

        Self {
            node_id,
            nodes,
            is_leader,
            term: 1,
        }
    }

    pub async fn propose_leader(&mut self, node_id: usize) -> Result<bool, String> {
        // Simple implementation - no real consensus, just setting leader
        self.is_leader = self.node_id == node_id;
        Ok(self.is_leader)
    }

    pub fn is_leader(&self) -> bool {
        self.is_leader
    }

    pub async fn heartbeat(&mut self) -> Result<(), String> {
        // No-op in simple implementation
        Ok(())
    }
}

impl Scheduler {
    pub async fn new(amqp_url: &str, storage: Arc<dyn WorkflowStorage>) -> Result<Arc<Self>, String> {
        // Initialize consensus with this node as ID 1
        let consensus = SimpleConsensus::new(1, vec![
            "127.0.0.1:8001".to_string(),
            "127.0.0.1:8002".to_string(),
            "127.0.0.1:8003".to_string(),
        ]);

        Ok(Arc::new(Self {
            workers: Arc::new(RwLock::new(HashMap::new())),
            task_queue: Arc::new(RwLock::new(None)), // Placeholder
            consensus: Arc::new(RwLock::new(consensus)),
            storage,
        }))
    }

    // Rest of implementation remains mostly the same

    pub async fn start_leader_election(&self) -> Result<(), String> {
        let node_id = 1; // This node's ID
        let mut consensus = self.consensus.write().await;
        consensus.propose_leader(node_id).await?;

        if consensus.is_leader() {
            let consensus_clone = self.consensus.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(5));
                loop {
                    interval.tick().await;
                    let mut consensus = consensus_clone.write().await;
                    if let Err(e) = consensus.heartbeat().await {
                        eprintln!("Leader heartbeat failed: {}", e);
                    }
                }
            });
        }

        Ok(())
    }

    pub async fn schedule_workflow(&self, workflow: Workflow, instance: WorkflowInstance) -> Result<(), String> {
        // Only the leader should schedule new workflows
        let consensus = self.consensus.read().await;
        if !consensus.is_leader() {
            return Err("Not the leader node - cannot schedule workflow".to_string());
        }

        // Find initial steps (those with no dependencies)
        let initial_steps: Vec<&Step> = workflow.steps.iter()
            .filter(|step| step.dependencies.is_empty())
            .collect();

        for step in initial_steps {
            self.schedule_step(workflow.clone(), instance.id, step.id).await?;
        }

        Ok(())
    }

    // Rest of the implementation remains the same
    pub async fn schedule_step(&self, workflow: Workflow, instance_id: Uuid, step_id: Uuid) -> Result<(), String> {
        // Find the step
        let step = workflow.steps.iter()
            .find(|s| s.id == step_id)
            .ok_or_else(|| format!("Step not found: {}", step_id))?;

        // Create task for the worker
        let task = worker::WorkerTask {
            id: Uuid::new_v4(),
            step_id,
            instance_id,
            task_type: step.task_type.clone(),
            config: step.config.clone(),
        };

        // Publish to task queue (in a real implementation)
        // self.publish_task(&task).await?;

        Ok(())
    }

    pub async fn reschedule_step(&self, workflow: Workflow, instance_id: Uuid, step_id: Uuid) -> Result<(), String> {
        // Similar to schedule_step, but would include logic to reset dependent steps
        self.schedule_step(workflow, instance_id, step_id).await
    }

    pub async fn register_worker(&self, worker_id: String, capabilities: Vec<TaskType>) -> Result<(), String> {
        let mut workers = self.workers.write().await;
        workers.insert(worker_id.clone(), WorkerInfo {
            id: worker_id,
            last_heartbeat: chrono::Utc::now(),
            capabilities,
        });
        Ok(())
    }

    pub async fn worker_heartbeat(&self, worker_id: &str) -> Result<(), String> {
        let mut workers = self.workers.write().await;
        if let Some(worker) = workers.get_mut(worker_id) {
            worker.last_heartbeat = chrono::Utc::now();
            Ok(())
        } else {
            Err(format!("Worker not found: {}", worker_id))
        }
    }

    pub async fn monitor_workers(&self) -> Result<(), String> {
        let mut workers = self.workers.write().await;
        let now = chrono::Utc::now();
        let timeout = chrono::Duration::seconds(30);

        // Find dead workers
        let dead_workers: Vec<String> = workers.iter()
            .filter(|(_, info)| now - info.last_heartbeat > timeout)
            .map(|(id, _)| id.clone())
            .collect();

        // Remove dead workers
        for id in &dead_workers {
            workers.remove(id);
        }

        // Reschedule tasks from dead workers (would be implemented in a real system)

        Ok(())
    }

    pub async fn process_feedback(&self, instance_id: Uuid, step_id: Uuid) -> Result<(), String> {
        // Implementation remains the same
        let instance = self.storage.get_instance(instance_id).await.map_err(|e| e.to_string())?;
        let workflow = self.storage.get_workflow(instance.workflow_id).await.map_err(|e| e.to_string())?;

        // Find feedback loops starting from this step
        for feedback in &workflow.feedback_loops {
            if feedback.source_step_id == step_id {
                // Implementation for processing feedback loops
                let should_trigger = true; // Simplified for this example
                if should_trigger {
                    self.reschedule_step(workflow.clone(), instance.id, feedback.target_step_id).await?;
                }
            }
        }

        Ok(())
    }
}