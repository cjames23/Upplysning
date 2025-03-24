use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::model::feedbackcondition::FeedbackCondition;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackLoop {
    pub source_step_id: Uuid,
    pub target_step_id: Uuid,
    pub condition: FeedbackCondition,
}