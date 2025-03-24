use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::model::feedbackloop::FeedbackLoop;
use crate::model::step::Step;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: Uuid,
    pub name: String,
    pub steps: Vec<Step>,
    pub feedback_loops: Vec<FeedbackLoop>,
}