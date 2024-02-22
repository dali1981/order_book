use std::collections::BTreeMap;
use rust_decimal::Decimal;

pub struct OrderBook {
    bids: BTreeMap<Decimal, Decimal>,
    asks: BTreeMap<Decimal, Decimal>,
}

impl Clone for OrderBook {
    fn clone(&self) -> Self {
        OrderBook {  bids: self.bids.clone(), asks:self.asks.clone() }
    }
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn update(&mut self, update: &OrderBookUpdate){
        update.price_levels.iter().for_each(|price_level| {

            let price = price_level.price;
            let quantity = price_level.quantity;
            // let level_key = format!("{:.10}", price);

            match price_level.quote_type {
                QuoteType::BID => {
                    self.update_bid(price, quantity);
                },
                QuoteType::ASK => {
                    self.update_ask(price, quantity);
                },
            }
        }
        )
    }

    pub fn update_ask(&mut self, price: Decimal, quantity: Decimal) {
        if quantity == Decimal::ZERO {
            // If the quantity is zero, remove the price level from the asks
            self.asks.remove(&price);
        } else {
            // Otherwise, insert or update the price level with the new quantity
            self.asks.insert(price, quantity);
        }
    }

    pub fn update_bid(&mut self, price: Decimal, quantity: Decimal) {
        if quantity == Decimal::ZERO  {
            // If the quantity is zero, remove the price level from the bids
            self.bids.remove(&price);
        } else {
            // Otherwise, insert or update the price level with the new quantity
            self.bids.insert(price, quantity);
        }
    }


    // Best bid is the last key in bids, because it's the highest
    pub fn best_bid(&self) -> Option<(Decimal, Decimal)> {
        self.bids.iter().rev().next().map(|(&price, &volume)| (price, volume))
    }

    // Best ask is the first key in asks, because it's the lowest
    pub fn best_ask(&self) -> Option<(Decimal, Decimal)> {
        self.asks.iter().next().map(|(&price, &volume)| (price, volume))
    }

    pub fn best(&self) -> () {
        let best_bid = self.best_bid()
            .map(|(price, volume)| format!("{:.8}, {:.2}", volume, price))
            .unwrap_or_else(|| "No bids".to_string());

        let best_ask = self.best_ask()
            .map(|(price, volume)| format!("{:.2}, {:.8}", price, volume))
            .unwrap_or_else(|| "No asks".to_string());

        println!("bid: {}, ask: {}", best_bid, best_ask);
    }

    // pub fn best_quotes(&self) -> Option<(PriceLevel, PriceLevel)> {
    //     let best_ask = self.best_ask();
    //     let best_bid = self.best_bid();
    //     match (best_bid, best_ask) {
    //         (Some(a), Some(b)) => Some((a, b)),
    //         _ => None,
    //     }
    // }
}

pub trait OrderBookEvent {
    fn update(&self, order_book: OrderBook);
}

pub enum QuoteType{
    BID, ASK
}

pub struct PriceLevel {
    price: Decimal,
    quantity: Decimal,
    quote_type: QuoteType
}

impl PriceLevel {
    pub fn new(price: Decimal, quantity: Decimal, quote_type: QuoteType) -> Self {
        PriceLevel{price, quantity, quote_type}
    }
}

pub struct OrderBookUpdate {
    price_levels: Vec<PriceLevel>
}

impl OrderBookUpdate {
    pub fn new(price_levels: Vec<PriceLevel>) -> Self {
        OrderBookUpdate{price_levels}
    }
}


impl OrderBookUpdate {

}