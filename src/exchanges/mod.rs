pub mod binance;
pub mod hyperliquid;
pub mod gateio;

use std::sync::Arc;
use crate::types::{ PriceUpdate, ExchangeArg };

use binance::Binance;
use hyperliquid::Hyperliquid;
use gateio::GateIo;

use async_trait::async_trait;
use tokio::sync::mpsc;
use std::error::Error;

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