use super::from_websocket;
use super::Exchange;
use crate::types::PriceUpdate;

use async_trait::async_trait;
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use std::error::Error;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub struct GateIo;

#[derive(Deserialize, Debug)]
struct WebsocketResponse {
    result: MarketData,
}

#[derive(Deserialize, Debug)]
struct MarketData {
    #[serde(default, deserialize_with = "from_websocket")]
    highest_bid: Option<f64>,
    #[serde(default, deserialize_with = "from_websocket")]
    lowest_ask: Option<f64>,
}

#[async_trait]
impl Exchange for GateIo {
    fn name(&self) -> &str {
        "Gate.io"
    }

    async fn websocket_subscribe(
        &self,
        tx: mpsc::Sender<PriceUpdate>,
        ticker: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
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
                let response: WebsocketResponse = serde_json::from_str(&text)?;

                if let (Some(bid), Some(ask)) =
                    (response.result.highest_bid, response.result.lowest_ask)
                {
                    let update = PriceUpdate::new("Gate.io".to_string(), bid, ask);
                    tx.send(update).await?;
                }
            }
        }

        Ok(())
    }
}
