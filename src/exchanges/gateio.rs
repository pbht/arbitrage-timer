use crate::types::PriceUpdate;
use super::Exchange;

use std::error::Error;
use async_trait::async_trait;
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub struct GateIo;

#[async_trait]
impl Exchange for GateIo {
    fn name(&self) ->  &str {
        "Gate.io"
    }

    async fn websocket_subscribe(&self, tx: mpsc::Sender<PriceUpdate>, ticker: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let url = "wss://api.gateio.ws/ws/v4/";
        let payload = json!({
            "time": Utc::now().timestamp(),
            "channel": "spot.tickers",
            "event": "subscribe",
            "payload": vec![format!("{}_USDT", ticker.to_uppercase())]
        });

        let (mut ws_stream, _) = connect_async(url).await?;
        ws_stream.send(Message::Text(payload.to_string())).await?;
        println!("Connected to Gate.io Websocket");

        let (_, mut read) = ws_stream.split();

        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text)) = msg {
                let json: Value = serde_json::from_str(&text)?;

                if json["event"] == "update".to_string() {
                    let bid = json["result"]["highest_bid"].as_str().unwrap().parse::<f64>()?;
                    let ask = json["result"]["lowest_ask"].as_str().unwrap().parse::<f64>()?;

                    let update = PriceUpdate::new("Gate.io".to_string(), bid, ask);

                    tx.send(update).await?;
                }
            }
        }

        Ok(())
    }
}