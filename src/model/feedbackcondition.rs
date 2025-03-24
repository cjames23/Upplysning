use serde::{Deserialize, Serialize};
use crate::model::thresholdoperator::ThresholdOperator;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeedbackCondition {
    MetricThreshold { metric: String, threshold: f64, operator: ThresholdOperator },
    Manual,
    Custom(String),
}