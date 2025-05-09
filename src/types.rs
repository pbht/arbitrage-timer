use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(long, num_args=2)]
    pub exchanges: Vec<String>,

    #[arg(long)]
    pub ticker: String,

    #[arg(long, default_value_t = 0.005)]
    pub threshold: f64

}

#[derive(Debug)]
pub struct PriceUpdate {
    pub exchange: String,
    pub bid: f64,
    pub ask: f64,
}


impl PriceUpdate {
    pub fn new(exchange: String, bid: f64, ask: f64) -> PriceUpdate {
        PriceUpdate { exchange, bid, ask }
    }
    
}

#[derive(Debug)]
pub struct CombinedPriceData {
    exchange_0: Option<PriceUpdate>,
    exchange_1: Option<PriceUpdate>
}

impl CombinedPriceData {
    pub fn new() -> Self {
        CombinedPriceData { exchange_0: None, exchange_1: None }
    }

    pub fn set_exchange_0(&mut self, update_0: PriceUpdate) -> &mut Self {
        self.exchange_0 = Some(update_0);
        self
    }

    pub fn set_exchange_1(&mut self, update_1: PriceUpdate) -> &mut Self {
        self.exchange_1 = Some(update_1);
        self
    }

    pub fn get_exchange_0(&self) -> Option<&PriceUpdate> {
        self.exchange_0.as_ref()
    }

    pub fn get_exchange_1(&self) -> Option<&PriceUpdate> {
        self.exchange_1.as_ref()
    }
}
