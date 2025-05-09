use std::error::Error;
use async_trait::async_trait;
use futures_util::StreamExt;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::types::PriceUpdate;
use super::Exchange;

pub struct Binance;

#[async_trait]
impl Exchange for Binance {
    fn name(&self) -> &str {
        "Binance"
    }

    async fn websocket_subscribe(&self, tx: mpsc::Sender<PriceUpdate>, ticker: &str) -> Result<(), Box<dyn Error + Send + Sync>> {

        let url = format!("wss://stream.binance.com:9443/ws/{}usdt@bookTicker", ticker.to_lowercase());

        let (ws_stream, _) = connect_async(url).await?;
        println!("Connected to Binance Websocket");

        let (_, mut read) = ws_stream.split();

        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text)) = msg {
                let json: Value = serde_json::from_str(&text)?;

                let bid = json["b"].as_str().unwrap().parse::<f64>()?;
                let ask = json["a"].as_str().unwrap().parse::<f64>()?;
                
                let update = PriceUpdate::new("Binance".to_string(), bid, ask);
                
                tx.send(update).await?;
            }

        }

        Ok(())
    }
}