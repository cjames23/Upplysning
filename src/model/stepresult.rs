use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::model::stepstatus::StepStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_id: Uuid,
    pub status: StepStatus,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub worker_id: Option<String>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}