use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade}, State},
    response::IntoResponse,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde_json::json;
use std::sync::Arc;
use tokio::time::{Duration, interval};
use tracing::{info, error};
use crate::MonitoringService;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(service): State<Arc<MonitoringService>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, service))
}

async fn handle_socket(socket: WebSocket, service: Arc<MonitoringService>) {
    let (mut sender, mut receiver) = socket.split();
    
    info!("WebSocket client connected");

    // Send initial cluster status
    if let Ok(msg) = serde_json::to_string(&json!({
        "type": "initial",
        "data": service.get_cluster_status().await
    })) {
        let _ = sender.send(axum::extract::ws::Message::Text(msg)).await;
    }

    // Create interval for periodic updates
    let mut ticker = interval(Duration::from_secs(5));

    // Handle incoming messages and send periodic updates
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                // Send periodic cluster status update
                let status = service.get_cluster_status().await;
                let message = json!({
                    "type": "update",
                    "timestamp": chrono::Utc::now(),
                    "data": status
                });

                if let Ok(msg) = serde_json::to_string(&message) {
                    if sender.send(axum::extract::ws::Message::Text(msg)).await.is_err() {
                        break;
                    }
                }
            }
            
            msg = receiver.next() => {
                match msg {
                    Some(Ok(axum::extract::ws::Message::Text(text))) => {
                        // Handle client messages
                        if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                            handle_client_message(client_msg, &service, &mut sender).await;
                        }
                    }
                    Some(Ok(axum::extract::ws::Message::Close(_))) => {
                        info!("WebSocket client disconnected");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}

#[derive(serde::Deserialize)]
struct ClientMessage {
    #[serde(rename = "type")]
    msg_type: String,
    data: Option<serde_json::Value>,
}

async fn handle_client_message(
    msg: ClientMessage,
    service: &Arc<MonitoringService>,
    sender: &mut futures_util::stream::SplitSink<WebSocket, axum::extract::ws::Message>,
) {
    match msg.msg_type.as_str() {
        "ping" => {
            let pong = json!({
                "type": "pong",
                "timestamp": chrono::Utc::now()
            });
            
            if let Ok(msg) = serde_json::to_string(&pong) {
                let _ = sender.send(axum::extract::ws::Message::Text(msg)).await;
            }
        }
        
        "get_agent" => {
            if let Some(data) = msg.data {
                if let Some(agent_id) = data.get("agent_id").and_then(|v| v.as_str()) {
                    let agents = service.agents.read().await;
                    if let Some(agent) = agents.get(agent_id) {
                        let response = json!({
                            "type": "agent_data",
                            "data": agent
                        });
                        
                        if let Ok(msg) = serde_json::to_string(&response) {
                            let _ = sender.send(axum::extract::ws::Message::Text(msg)).await;
                        }
                    }
                }
            }
        }
        
        "get_events" => {
            let events = service.events.read().await;
            let recent_events: Vec<_> = events.iter().rev().take(50).cloned().collect();
            
            let response = json!({
                "type": "events",
                "data": recent_events
            });
            
            if let Ok(msg) = serde_json::to_string(&response) {
                let _ = sender.send(axum::extract::ws::Message::Text(msg)).await;
            }
        }
        
        "subscribe_agent" => {
            // TODO: Implement agent-specific subscriptions
            if let Some(data) = msg.data {
                if let Some(agent_id) = data.get("agent_id").and_then(|v| v.as_str()) {
                    info!("Client subscribed to agent: {}", agent_id);
                }
            }
        }
        
        _ => {
            error!("Unknown message type: {}", msg.msg_type);
        }
    }
}