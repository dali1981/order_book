use serde_json::Value;
use crate::messages::IncomingMsg;
use crate::quote::Exchange::Kraken;

#[derive(Debug, PartialEq)]
pub enum Exchange {
    Kraken,
    Binance,
}

#[derive(Debug, PartialEq)]
pub struct Quote {
    pub exchange: Exchange,
    pub best_bid: f64,
    pub best_ask: f64,
}

impl Quote {
    fn new(exchange: Exchange, best_bid: f64, best_ask: f64) -> Self {
        Quote {
            exchange,
            best_bid,
            best_ask,
        }
    }

    pub fn parse(message: &IncomingMsg) -> Option<Self> {
        // Mock parsing logic, replace with actual JSON parsing

        match &message.exchange {
            Kraken => Self::parse_kraken(&message.msg),
            Exchange::Binance => Self::parse_binance(&message.msg),
            _ => None,
        }


    }

    fn parse_kraken(message: &str) -> Option<Self> {
        // Parse the JSON message
        let json_data: Result<Value, _> = serde_json::from_str(message);
        if let Ok(data) = json_data {
            // Extract bid and ask prices from the JSON
            if let (Some(bid), Some(ask)) = (data["bid"].as_f64(), data["ask"].as_f64()) {
                // Create and return a new Quote instance
                return Some(Quote::new(Exchange::Kraken, bid, ask));
            }
        }
        None
    }

    fn parse_binance(message: &str) -> Option<Self> {
        // Parse the JSON message
        let json_data: Result<Value, _> = serde_json::from_str(message);
        if let Ok(data) = json_data {
            // Extract bid and ask prices from the JSON
            if let (Some(bid), Some(ask)) = (data["bids"][0][0].as_str().and_then(|x| x.parse::<f64>().ok()), data["asks"][0][0].as_str().and_then(|x| x.parse::<f64>().ok())) {
                // Create and return a new Quote instance
                return Some(Quote::new(Exchange::Binance, bid, ask));
            }
        }
        None
    }

}

fn compare_mid_prices(quote1: &Quote, quote2: &Quote) -> f64 {
    let mid_price1 = (quote1.best_bid + quote1.best_ask) / 2.0;
    let mid_price2 = (quote2.best_bid + quote2.best_ask) / 2.0;
    mid_price1 - mid_price2
}
