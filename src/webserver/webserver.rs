use std::sync::Arc;
use axum::{
    routing::{get, post},
    Router, Json, extract::Path,
    debug_handler,
};
use axum::extract::State;
use axum::response::IntoResponse;
use uuid::Uuid;
use crate::engine::workflowengine::WorkflowEngine;
use crate::model::workflow::Workflow;

pub struct ApiServer {
    engine: Arc<WorkflowEngine>,
}

#[debug_handler]
async fn create_workflow(
    State(engine): State<Arc<WorkflowEngine>>,
    Json(workflow): Json<Workflow>,
) -> impl IntoResponse {
    Json(serde_json::json!({"id": workflow.id.to_string()}))
}

#[debug_handler]
async fn get_workflow(
    State(engine): State<Arc<WorkflowEngine>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    Json(serde_json::json!({"id": id}))
}

#[debug_handler]
async fn start_workflow_instance(
    State(engine): State<Arc<WorkflowEngine>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    Json(serde_json::json!({"instance_id": Uuid::new_v4().to_string()}))
}

#[debug_handler]
async fn get_instance(
    State(engine): State<Arc<WorkflowEngine>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    Json(serde_json::json!({"id": id}))
}

#[debug_handler]
async fn trigger_feedback(
    State(engine): State<Arc<WorkflowEngine>>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    Json(serde_json::json!({"status": "feedback_triggered"}))
}
impl ApiServer {
    pub fn new(engine: Arc<WorkflowEngine>) -> Self {
        Self { engine }
    }

    pub async fn start(self, addr: &str) -> Result<(), String> {
        let engine = self.engine.clone();
        let app = Router::new()
            .route("/workflows", post(create_workflow))
            .route("/workflows/:id", get(get_workflow))
            .route("/workflows/:id/instances", post(start_workflow_instance))
            .route("/instances/:id", get(get_instance))
            .route("/instances/:id/feedback", post(trigger_feedback))
            .with_state(engine);

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| e.to_string())?;

        axum::serve(listener, app)
            .await
            .map_err(|e| e.to_string())
    }


}