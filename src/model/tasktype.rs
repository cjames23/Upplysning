use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    DataPreparation,
    ModelTraining,
    ModelEvaluation,
    ModelDeployment,
    Notification,
    Custom(String),
}