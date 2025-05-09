mod exchanges;
mod types;
mod utils;

use chrono::DateTime;
use chrono::Utc;
use clap::Parser;
use std::error::Error;
use std::sync::Arc;
use tokio::try_join;
use tokio::sync::mpsc;

use types::Args;
use types::PriceUpdate;
use types::CombinedPriceData;
use exchanges::Exchange;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let args = Args::parse();
    let exchanges: Vec<Arc<dyn Exchange>> = args.exchanges
        .iter()
        .filter_map(|exchange| exchanges::get_cex(&exchange))
        .collect::<Vec<Arc<dyn Exchange>>>();

    let ticker = args.ticker;

    let (tx, mut rx) = mpsc::channel::<PriceUpdate>(100);

    let mut iter = exchanges.into_iter();
    let exchange_0: Arc<dyn Exchange + Send + Sync> = iter.next().unwrap();
    let exchange_1: Arc<dyn Exchange + Send + Sync> = iter.next().unwrap();

    let h0 = tokio::spawn({
        let tx = tx.clone();
        let exchange_0 = exchange_0.clone();
        let ticker = ticker.clone();
        async move {
            if let Err(e) = exchange_0.websocket_subscribe(tx, &ticker).await {
                println!("{} WS error: {:?}", exchange_0.name(), e);
            }
        }
    });

    let h1 = tokio::spawn({
        let tx = tx.clone();
        let exchange_1 = exchange_1.clone();
        let ticker = ticker.clone();
        async move {
            if let Err(e) = exchange_1.websocket_subscribe(tx, &ticker).await {
                println!("{} WS error: {:?}", exchange_1.name(), e);
            }
        }
    });

    let mut data= CombinedPriceData::new();
    let name_0 = exchange_0.name();
    let name_1 = exchange_1.name();

    let arb_threshold = 1.0 + args.threshold;
    let mut arbitrage_start: Option<DateTime<Utc>> = None;

    while let Some(update) = rx.recv().await {
        // println!("[{}] : Bid: {:.2}, Ask: {:.2} @ {}", update.exchange, update.bid, update.ask, update.timestamp);
        if let (Some(e0), Some(e1)) = (data.get_exchange_0(), data.get_exchange_1()) {
            let is_arb = e0.bid > e1.ask * arb_threshold || e1.bid > e0.ask * arb_threshold;
        
            match (arbitrage_start.is_some(), is_arb) {
                (false, true) => {
                    // Arbitrage just started
                    arbitrage_start = Some(Utc::now());
                    println!("Arbitrage started: {}", arbitrage_start.unwrap());
                }
                (true, false) => {
                    // Arbitrage just ended
                    let started = arbitrage_start.take().unwrap(); // take the value and reset to None
                    let duration = Utc::now() - started;
                    println!(
                        "Arbitrage ended. Duration: {} ms",
                        duration.num_milliseconds()
                    );
                }
                _ => {}
            }
        }
        
        if update.exchange == name_0 {
            data.set_exchange_0(update);
        } else if update.exchange == name_1 {
            data.set_exchange_1(update);
        }
        
    }

    try_join!(h0, h1)?;

    Ok(())
}