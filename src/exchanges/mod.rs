pub mod binance;
pub mod gate_io;
pub mod hyperliquid;

use crate::types::{ExchangeArg, PriceUpdate};
use std::sync::Arc;

use binance::Binance;
use gate_io::GateIo;
use hyperliquid::Hyperliquid;

use async_trait::async_trait;
use serde::{Deserialize, Deserializer};
use std::error::Error;
use tokio::sync::mpsc;

// Helper function to parse websocket stream values
fn from_websocket<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        Some(s) => {
            let num = s.parse::<f64>().map_err(serde::de::Error::custom)?;
            Ok(Some(num))
        }
        None => Ok(None),
    }
}

#[async_trait]
pub trait Exchange: Send + Sync {
    fn name(&self) -> &str;
    async fn websocket_subscribe(
        &self,
        tx: mpsc::Sender<PriceUpdate>,
        ticker: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}

pub fn get_cex(name: &ExchangeArg) -> Arc<dyn Exchange> {
    match name {
        ExchangeArg::Binance => Arc::new(Binance),
        ExchangeArg::Hyperliquid => Arc::new(Hyperliquid),
        ExchangeArg::GateIo => Arc::new(GateIo),
    }
}
