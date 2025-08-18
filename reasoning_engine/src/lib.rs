use common::{AppEvent, EventReceiver, EventSender};
use tokio::sync::broadcast::error::RecvError;
use tracing::{error, info, warn};

pub struct ReasoningEngine {
    tx: EventSender,
    rx: EventReceiver,
}

impl ReasoningEngine {
    pub fn new(tx: EventSender, rx: EventReceiver) -> Self {
        Self { tx, rx }
    }

    pub async fn run(&mut self) {
        info!("[Reasoning Engine] Starting...");
        loop {
            match self.rx.recv().await {
                Ok(AppEvent::WebSearchQuery(query)) => self.handle_web_search(query).await,
                Ok(AppEvent::LlmQuery(query)) => self.handle_llm_query(query).await,
                Ok(_) => {}
                Err(RecvError::Lagged(n)) => warn!("[Reasoning Engine] Lagged by {} messages", n),
                Err(RecvError::Closed) => {
                    error!("[Reasoning Engine] Event channel closed.");
                    break;
                }
            }
        }
    }

    async fn handle_web_search(&self, query: String) {
        info!(
            "[Reasoning Engine] Received WebSearchQuery for: '{}'. Emitting simulated response.",
            query
        );
        // In a real human-in-the-loop or agent-driven system, the agent would see the log above
        // and call the google_web_search tool. For now, we simulate the agent's action.
        let results = vec![
            "https://www.coindesk.com/markets/2025/08/05/bitcoin-holds-steady-as-new-data-emerges/"
                .to_string(),
            "https://cointelegraph.com/news/analysis-bitcoin-price-prediction-2025".to_string(),
        ];
        let response = AppEvent::WebSearchResponse(results);
        if let Err(e) = self.tx.send(response) {
            error!("[Reasoning Engine] Failed to send WebSearchResponse: {}", e);
        }
    }

    async fn handle_llm_query(&self, url: String) {
        info!(
            "[Reasoning Engine] Received LlmQuery for URL: '{}'. Simulating fetch and analysis.",
            url
        );
        // The agent would see the log above, call the web_fetch tool, and then another LLM for analysis.
        // We simulate both actions.
        let fetched_content_snippet = "(Simulated Fetched Content) Bitcoin (BTC) remained stable on Tuesday morning, trading around the $70,000 mark as investors digested new inflation data...";
        info!(
            "[Reasoning Engine] Simulated fetched content: '{}'",
            fetched_content_snippet
        );

        let llm_response = "SIMULATED SENTIMENT: The fetched content appears to be neutral, with a focus on market stability.".to_string();
        let response = AppEvent::LlmResponse(llm_response);
        if let Err(e) = self.tx.send(response) {
            error!("[Reasoning Engine] Failed to send LlmResponse: {}", e);
        }
    }
}
