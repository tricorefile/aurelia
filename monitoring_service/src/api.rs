use axum::{
    extract::{Path, State, Query},
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::{MonitoringService, AgentStatus, ClusterStatus, ClusterEvent, ClusterHealth};

#[derive(Debug, Deserialize)]
pub struct MetricsUpdate {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub disk_usage: f32,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub tasks_completed: u32,
    pub tasks_failed: u32,
    pub replicas_active: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct EventQuery {
    pub limit: Option<usize>,
    pub severity: Option<String>,
    pub agent_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

// Get all agents
pub async fn get_agents(
    State(service): State<Arc<MonitoringService>>,
) -> impl IntoResponse {
    let agents = service.agents.read().await;
    let agent_list: Vec<AgentStatus> = agents.values().cloned().collect();
    
    Json(ApiResponse::success(agent_list))
}

// Get specific agent
pub async fn get_agent(
    Path(agent_id): Path<String>,
    State(service): State<Arc<MonitoringService>>,
) -> impl IntoResponse {
    let agents = service.agents.read().await;
    
    match agents.get(&agent_id) {
        Some(agent) => (StatusCode::OK, Json(ApiResponse::success(agent.clone()))),
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<AgentStatus>::error(format!("Agent {} not found", agent_id)))
        ),
    }
}

// Update agent metrics
pub async fn update_agent_metrics(
    Path(agent_id): Path<String>,
    State(service): State<Arc<MonitoringService>>,
    Json(metrics): Json<MetricsUpdate>,
) -> impl IntoResponse {
    let mut agents = service.agents.write().await;
    
    if let Some(agent) = agents.get_mut(&agent_id) {
        agent.cpu_usage = metrics.cpu_usage;
        agent.memory_usage = metrics.memory_usage;
        agent.disk_usage = metrics.disk_usage;
        agent.network_rx_bytes = metrics.network_rx_bytes;
        agent.network_tx_bytes = metrics.network_tx_bytes;
        agent.tasks_completed = metrics.tasks_completed;
        agent.tasks_failed = metrics.tasks_failed;
        agent.replicas_active = metrics.replicas_active;
        agent.last_heartbeat = chrono::Utc::now();
        
        if agent.cpu_usage < 10.0 {
            agent.status = crate::AgentState::Idle;
        } else if agent.cpu_usage > 80.0 {
            agent.status = crate::AgentState::Busy;
        } else {
            agent.status = crate::AgentState::Running;
        }
        
        (StatusCode::OK, Json(ApiResponse::success("Metrics updated")))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<&str>::error(format!("Agent {} not found", agent_id)))
        )
    }
}

// Get cluster status
pub async fn get_cluster_status(
    State(service): State<Arc<MonitoringService>>,
) -> impl IntoResponse {
    let status = service.get_cluster_status().await;
    Json(ApiResponse::success(status))
}

// Get events
pub async fn get_events(
    Query(query): Query<EventQuery>,
    State(service): State<Arc<MonitoringService>>,
) -> impl IntoResponse {
    let events = service.events.read().await;
    
    let mut filtered_events: Vec<ClusterEvent> = events.clone();
    
    // Filter by agent_id if provided
    if let Some(agent_id) = query.agent_id {
        filtered_events.retain(|e| e.agent_id == agent_id);
    }
    
    // Filter by severity if provided
    if let Some(severity) = query.severity {
        filtered_events.retain(|e| {
            format!("{:?}", e.severity).to_lowercase() == severity.to_lowercase()
        });
    }
    
    // Limit results
    let limit = query.limit.unwrap_or(100);
    filtered_events.truncate(limit);
    
    Json(ApiResponse::success(filtered_events))
}

// Get cluster health
pub async fn get_cluster_health(
    State(service): State<Arc<MonitoringService>>,
) -> impl IntoResponse {
    let status = service.get_cluster_status().await;
    
    #[derive(Serialize)]
    struct HealthResponse {
        health: ClusterHealth,
        total_agents: usize,
        healthy_agents: usize,
        degraded_agents: usize,
        offline_agents: usize,
        health_percentage: f32,
    }
    
    let health_percentage = if status.total_agents > 0 {
        (status.healthy_agents as f32 / status.total_agents as f32) * 100.0
    } else {
        0.0
    };
    
    let response = HealthResponse {
        health: status.cluster_health,
        total_agents: status.total_agents,
        healthy_agents: status.healthy_agents,
        degraded_agents: status.degraded_agents,
        offline_agents: status.offline_agents,
        health_percentage,
    };
    
    Json(ApiResponse::success(response))
}