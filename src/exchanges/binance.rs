use async_trait::async_trait;
use futures_util::StreamExt;
use serde::Deserialize;
use std::error::Error;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use super::from_websocket;
use super::Exchange;
use crate::types::PriceUpdate;

pub struct Binance;

#[derive(Debug, Deserialize)]
struct WebsocketResponse {
    #[serde(default, deserialize_with = "from_websocket")]
    b: Option<f64>,
    #[serde(default, deserialize_with = "from_websocket")]
    a: Option<f64>,
}

#[async_trait]
impl Exchange for Binance {
    fn name(&self) -> &str {
        "Binance"
    }

    async fn websocket_subscribe(
        &self,
        tx: mpsc::Sender<PriceUpdate>,
        ticker: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let url = format!(
            "wss://stream.binance.com:9443/ws/{}usdt@bookTicker",
            ticker.to_lowercase()
        );

        let (ws_stream, _) = connect_async(url).await?;
        println!("Connected to Binance Websocket");

        let (_, mut read) = ws_stream.split();

        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text)) = msg {
                let response: WebsocketResponse = serde_json::from_str(&text)?;

                if let (Some(bid), Some(ask)) = (response.b, response.a) {
                    let update = PriceUpdate::new("Binance".to_string(), bid, ask);
                    tx.send(update).await?;
                }
            }
        }

        Ok(())
    }
}
