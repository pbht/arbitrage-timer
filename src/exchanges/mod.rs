pub mod binance;
pub mod hyperliquid;

use std::sync::Arc;
use binance::Binance;
use hyperliquid::Hyperliquid;
use crate::types::PriceUpdate;

use async_trait::async_trait;
use tokio::sync::mpsc;
use std::error::Error;

#[async_trait]
pub trait Exchange: Send + Sync {
    fn name(&self) -> &str;
    async fn websocket_subscribe(&self, tx: mpsc::Sender<PriceUpdate>, ticker: &str) -> Result<(), Box<dyn Error + Send + Sync>>;
}

pub fn get_cex(name: &str) -> Option<Arc<dyn Exchange>> {
    match name {
        "binance" => Some(Arc::new(Binance)),
        "hyperliquid" => Some(Arc::new(Hyperliquid)),
        _ => None
    }
}