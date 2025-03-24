use std::sync::Arc;
use Upplysning::{engine, storage};
use Upplysning::engine::workflowengine::WorkflowEngine;
use Upplysning::scheduler::scheduler;
use Upplysning::storage::workflowstorage::InMemoryStorage;
use Upplysning::webserver::webserver::ApiServer;

#[tokio::main]
pub async fn main() -> Result<(), String> {
    // Initialize storage
    let storage = Arc::new(InMemoryStorage::new());

    // Initialize scheduler with Paxos for leader election
    let scheduler = scheduler::Scheduler::new("amqp://localhost", storage.clone()).await?;

    // Start leader election
    scheduler.start_leader_election().await?;

    // Initialize engine
    let engine = Arc::new(WorkflowEngine::new(scheduler.clone(), storage.clone()));

    // Start API server
    let api_server = ApiServer::new(engine);
    api_server.start("0.0.0.0:3000").await?;

    Ok(())
}