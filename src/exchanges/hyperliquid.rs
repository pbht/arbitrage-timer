use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use std::error::Error;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use super::from_websocket;
use super::Exchange;
use crate::types::PriceUpdate;

pub struct Hyperliquid;

#[derive(Debug, Deserialize)]
struct WebsocketResponse {
    data: WebsocketResponseData,
}

#[derive(Debug, Deserialize)]
struct WebsocketResponseData {
    levels: Option<Vec<Vec<WebsocketPrice>>>,
}

#[derive(Debug, Deserialize)]
struct WebsocketPrice {
    #[serde(default, deserialize_with = "from_websocket")]
    px: Option<f64>,
}

#[async_trait]
impl Exchange for Hyperliquid {
    fn name(&self) -> &str {
        "Hyperliquid"
    }

    async fn websocket_subscribe(
        &self,
        tx: mpsc::Sender<PriceUpdate>,
        ticker: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let url = "wss://api.hyperliquid.xyz/ws";
        let payload = json!({
            "method": "subscribe",
            "subscription": {
                "type": "l2Book",
                "coin": ticker
            }
        });

        let (mut ws_stream, _) = connect_async(url).await?;
        ws_stream.send(Message::Text(payload.to_string())).await?;
        println!("Connected to Hyperliquid Websocket");

        let (_, mut read) = ws_stream.split();

        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text)) = msg {
                let response: WebsocketResponse = serde_json::from_str(&text)?;

                if let Some(levels) = response.data.levels {
                    let bids = levels.first().ok_or("Error obtaining bids orderbook")?;
                    let asks = levels.last().ok_or("Error obtaining asks orderbook")?;

                    let best_bid = bids.first();
                    let best_ask = asks.first();

                    if let (Some(bid), Some(ask)) = (best_bid, best_ask) {
                        if let (Some(bid), Some(ask)) = (bid.px, ask.px) {
                            let update = PriceUpdate::new("Hyperliquid".to_string(), bid, ask);
                            tx.send(update).await?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
