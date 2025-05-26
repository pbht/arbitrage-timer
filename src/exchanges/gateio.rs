use crate::types::PriceUpdate;
use super::Exchange;

use std::error::Error;
use async_trait::async_trait;
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Deserializer};
use serde_json::json;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub struct GateIo;

#[derive(Deserialize, Debug)]
struct GateIoResponse {
    result: GateIoMarketData
}

#[derive(Deserialize, Debug)]
struct GateIoMarketData {
    #[serde(default, deserialize_with="from_str")]
    highest_bid: Option<f64>,
    #[serde(default, deserialize_with="from_str")]
    lowest_ask: Option<f64>
}

fn from_str<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error> 
where 
    D: Deserializer<'de>
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        Some(s) => {
            let num = s.parse::<f64>().map_err(serde::de::Error::custom)?;
            Ok(Some(num))
        }
        None => Ok(None)
    }
}

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

                let response: GateIoResponse = serde_json::from_str(&text)?;

                if let (Some(bid), Some(ask)) = (response.result.highest_bid, response.result.lowest_ask) {
                    let update = PriceUpdate::new("Gate.io".to_string(), bid, ask);
                    tx.send(update).await?;
                }
            }
        }

        Ok(())
    }
}