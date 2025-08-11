use futures_util::{pin_mut, stream::StreamExt};
use rustls::crypto::CryptoProvider;
use serde::Deserialize;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use common::{AppEvent, EventSender, MarketData};

#[derive(Debug, Deserialize)]
pub struct BinanceTrade {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "p")]
    pub price: String,
    #[serde(rename = "q")]
    pub quantity: String,
    #[serde(rename = "T")]
    pub timestamp: u64,
}

const BINANCE_WS_API: &str = "wss://stream.binance.com:9443/ws/btcusdt@trade";

pub async fn run(tx: EventSender) {
    let _ = CryptoProvider::install_default(rustls::crypto::ring::default_provider());

    println!("[Perception Core] Connecting to Binance WebSocket...");

    let (ws_stream, _) = connect_async(BINANCE_WS_API)
        .await
        .expect("Failed to connect to WebSocket");
    tracing::info!("[Perception Core] Connection to Binance WebSocket successful. Awaiting market data...");

    let (_write, read) = ws_stream.split();
    pin_mut!(read);

    while let Some(message) = read.next().await {
        if let Ok(Message::Text(text)) = message {
            if let Ok(trade) = serde_json::from_str::<BinanceTrade>(&text) {
                let market_data = MarketData {
                    symbol: trade.symbol,
                    price: trade.price.parse().unwrap_or(0.0),
                    quantity: trade.quantity.parse().unwrap_or(0.0),
                    timestamp: trade.timestamp,
                };
                if let Err(e) = tx.send(AppEvent::MarketData(market_data)) {
                    eprintln!("[Perception Core] Failed to send market data: {}", e);
                }
            }
        }
    }
}