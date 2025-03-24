use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::model::tasktype::TaskType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub id: Uuid,
    pub name: String,
    pub task_type: TaskType,
    pub config: HashMap<String, String>,
    pub dependencies: Vec<Uuid>,
}