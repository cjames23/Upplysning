use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use crate::model::feedbackcondition::FeedbackCondition;
use crate::model::feedbackloop::FeedbackLoop;
use crate::model::instancestatus::InstanceStatus;
use crate::model::thresholdoperator::ThresholdOperator;
use crate::scheduler::scheduler;
use crate::model::workflow::Workflow;
use crate::model::workflowinstance::WorkflowInstance;
use crate::storage::workflowstorage::WorkflowStorage;

pub struct WorkflowEngine {
    scheduler: Arc<scheduler::Scheduler>,
    storage: Arc<dyn WorkflowStorage>,
}

impl WorkflowEngine {
    pub fn new(scheduler: Arc<scheduler::Scheduler>, storage: Arc<dyn WorkflowStorage>) -> Self {
        Self { scheduler, storage }
    }

    pub async fn start_workflow(&self, workflow: Workflow) -> Result<Uuid, String> {
        let instance_id = Uuid::new_v4();
        let instance = WorkflowInstance {
            id: instance_id,
            workflow_id: workflow.id,
            status: InstanceStatus::Pending,
            step_results: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.storage.save_workflow(&workflow).await.map_err(|e| e.to_string())?;
        self.storage.save_instance(&instance).await.map_err(|e| e.to_string())?;

        self.scheduler.schedule_workflow(workflow, instance).await?;

        Ok(instance_id)
    }

    pub async fn process_feedback(&self, instance_id: Uuid, step_id: Uuid) -> Result<(), String> {
        let instance = self.storage.get_instance(instance_id).await.map_err(|e| e.to_string())?;
        let workflow = self.storage.get_workflow(instance.workflow_id).await.map_err(|e| e.to_string())?;

        // Find feedback loops starting from this step
        for feedback in &workflow.feedback_loops {
            if feedback.source_step_id == step_id {
                // Check if condition is met
                let should_trigger = self.evaluate_feedback_condition(&instance, &feedback).await?;
                if should_trigger {
                    // Re-execute from target step
                    self.scheduler.reschedule_step(workflow.clone(), instance.id, feedback.target_step_id).await?;
                }
            }
        }

        Ok(())
    }

    async fn evaluate_feedback_condition(&self, instance: &WorkflowInstance, feedback: &FeedbackLoop) -> Result<bool, String> {
        match &feedback.condition {
            FeedbackCondition::MetricThreshold { metric, threshold, operator } => {
                if let Some(result) = instance.step_results.get(&feedback.source_step_id) {
                    if let Some(output) = &result.output {
                        if let Some(metric_value) = output.get(metric) {
                            if let Some(value) = metric_value.as_f64() {
                                return Ok(match operator {
                                    ThresholdOperator::GreaterThan => value > *threshold,
                                    ThresholdOperator::LessThan => value < *threshold,
                                    ThresholdOperator::Equal => (value - threshold).abs() < f64::EPSILON,
                                });
                            }
                        }
                    }
                }
                Ok(false)
            },
            FeedbackCondition::Manual => Ok(true), // Manual feedback is always triggered when requested
            FeedbackCondition::Custom(code) => {
                // In a real system, we'd have a scripting engine to evaluate custom conditions
                Ok(false)
            }
        }
    }
}
