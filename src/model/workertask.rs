use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::model::tasktype::TaskType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerTask {
    pub id: Uuid,
    pub step_id: Uuid,
    pub instance_id: Uuid,
    pub task_type: TaskType,
    pub config: HashMap<String, String>,
}