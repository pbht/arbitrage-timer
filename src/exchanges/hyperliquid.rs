use std::error::Error;
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};


use crate::types::PriceUpdate;
use super::Exchange;
use super::from_websocket;


pub struct Hyperliquid;

#[derive(Debug, Deserialize)]
struct WebsocketResponse {
    data: WebsocketResponseData
}

#[derive(Debug, Deserialize)]
struct WebsocketResponseData {
    levels: Option<Vec<Vec<WebsocketPrice>>>
}

#[derive(Debug, Deserialize)]
struct WebsocketPrice {
    #[serde(default, deserialize_with="from_websocket")]
    px: Option<f64>
}

#[async_trait]
impl Exchange for Hyperliquid {
    fn name(&self) -> &str {
        "Hyperliquid"
    }

    async fn websocket_subscribe(&self, tx: mpsc::Sender<PriceUpdate>, ticker: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        
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
                    let bids  = levels.get(0).ok_or_else(|| "Error obtaining bids orderbook")?;
                    let asks = levels.get(1).ok_or_else(|| "Error obtaining asks orderbook")?;
                    
                    let best_bid = bids.get(0);
                    let best_ask = asks.get(0);
    
                    match (best_bid, best_ask) {
                        (Some(bid), Some(ask)) => {
                            if let (Some(bid), Some(ask)) = (bid.px, ask.px) {
                                let update = PriceUpdate::new("Hyperliquid".to_string(), bid, ask);
                                tx.send(update).await?;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }


        Ok(())
    }
}
