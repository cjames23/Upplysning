use std::sync::Arc;
use tokio::sync::mpsc;
use crate::scheduler::scheduler;
use crate::model::tasktype::TaskType;
use crate::model::workertask::WorkerTask;
use crate::storage::workflowstorage::WorkflowStorage;

pub struct Worker {
    pub(crate) id: String,
    pub(crate) capabilities: Vec<TaskType>,
    pub(crate) scheduler: Arc<scheduler::Scheduler>,
    pub(crate) storage: Arc<dyn WorkflowStorage>,
    pub(crate) task_receiver: mpsc::Receiver<WorkerTask>,
}