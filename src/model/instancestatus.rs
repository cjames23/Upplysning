use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstanceStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}