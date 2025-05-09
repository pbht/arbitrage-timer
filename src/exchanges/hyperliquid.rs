use std::error::Error;
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};


use crate::types::PriceUpdate;
use super::Exchange;


pub struct Hyperliquid;


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
                let json: Value = serde_json::from_str(&text)?;

                if json["channel"] == "subscriptionResponse".to_string() {
                    continue;
                }

                let bid = json["data"]["levels"][0][0]["px"].as_str().unwrap().parse::<f64>()?;
                let ask = json["data"]["levels"][1][0]["px"].as_str().unwrap().parse::<f64>()?;

                let update = PriceUpdate::new("Hyperliquid".to_string(), bid, ask);

                tx.send(update).await?;
            }
        }


        Ok(())
    }
}
