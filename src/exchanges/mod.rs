pub mod binance;
pub mod hyperliquid;
pub mod gate_io;

use std::sync::Arc;
use crate::types::{ PriceUpdate, ExchangeArg };

use binance::Binance;
use hyperliquid::Hyperliquid;
use gate_io::GateIo;

use async_trait::async_trait;
use tokio::sync::mpsc;
use std::error::Error;
use serde::{Deserialize, Deserializer};

// Helper function to parse websocket stream values
fn from_websocket<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error> 
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
pub trait Exchange: Send + Sync {
    fn name(&self) -> &str;
    async fn websocket_subscribe(&self, tx: mpsc::Sender<PriceUpdate>, ticker: &str) -> Result<(), Box<dyn Error + Send + Sync>>;
}

pub fn get_cex(name: &ExchangeArg) -> Option<Arc<dyn Exchange>> {
    match name {
        ExchangeArg::Binance => Some(Arc::new(Binance)),
        ExchangeArg::Hyperliquid => Some(Arc::new(Hyperliquid)),
        ExchangeArg::GateIo => Some(Arc::new(GateIo))
    }
}