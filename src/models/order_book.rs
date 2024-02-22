use std::collections::BTreeMap;
use kraken_ws_client::api::{BookData, LevelData};
use rust_decimal::Decimal;
use std::string::String;

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


    fn update_level(side: &mut BTreeMap<Decimal, Decimal>, price: Decimal, quantity: Decimal) {
        if quantity == Decimal::ZERO {
            // If the quantity is zero, remove the price level from the bids
            side.remove(&price);
        } else {
            // Otherwise, insert or update the price level with the new quantity
            side.insert(price, quantity);
        }
    }

    pub fn update_from_kraken(&mut self, update: &Vec<BookData>) {
        update.iter().for_each(|book_data| {
            book_data.asks.iter().for_each(|level_data| {
                OrderBook::update_level(&mut self.asks, level_data.price, level_data.qty)
            });
            book_data.bids.iter().for_each(|level_data| {
                OrderBook::update_level(&mut self.bids, level_data.price, level_data.qty)
            });
        });
    }

    pub fn ensure_book_is_valid(&mut self) {
        // let best_ask = self.best_ask()?;
        // let best_bid = self.best_bid()?;

        panic!("not implemented")
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
        if quantity == Decimal::ZERO {
            // If the quantity is zero, remove the price level from the bids
            self.bids.remove(&price);
        } else {
            // Otherwise, insert or update the price level with the new quantity
            self.bids.insert(price, quantity);
        }
    }

    pub fn update(&mut self, update: &OrderBookUpdate) {
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

    // Best bid is the last key in bids, because it's the highest
    pub fn best_bid(&self) -> Option<(Decimal, Decimal)> {
        self.bids.iter().rev().next().map(|(&price, &volume)| (price, volume))
    }

    // Best ask is the first key in asks, because it's the lowest
    pub fn best_ask(&self) -> Option<(Decimal, Decimal)> {
        self.asks.iter().next().map(|(&price, &volume)| (price, volume))
    }

    pub fn best(&self) -> String {
        let best_bid = self.best_bid()
            .map(|(price, volume)| format!("{:.8}, {:.2}", volume, price))
            .unwrap_or_else(|| "No bids".to_string());

        let best_ask = self.best_ask()
            .map(|(price, volume)| format!("{:.2}, {:.8}", price, volume))
            .unwrap_or_else(|| "No asks".to_string());

        format!("bid: {}, ask: {}", best_bid, best_ask)
    }

    // New method to print order book in a tabular format
    pub fn print(&self) -> String {
        // Determine the longest side to ensure the table is even
        let bids_len = self.bids.len();
        let asks_len = self.asks.len();
        let max_len = std::cmp::max(bids_len, asks_len);

        // Collect bids and asks in vectors for easy indexing
        let mut bids_vec: Vec<_> = self.bids.iter().rev().collect(); // Reverse to start from the highest bid
        let mut asks_vec: Vec<_> = self.asks.iter().collect();

        // Pad the shorter side with empty tuples to match the length of the longer side
        while bids_vec.len() < max_len {
            bids_vec.push((&Decimal::ZERO, &Decimal::ZERO));
        }
        while asks_vec.len() < max_len {
            asks_vec.push((&Decimal::ZERO, &Decimal::ZERO));
        }

        // Print the header
        let mut out = format!("{:<15} {:<15} {:<15} {:<15}\n", "Bid", "Volume", "Ask", "Volume");

        // Print each row of the order book
        for i in 0..max_len {
            let (bid_price, bid_volume) = bids_vec[i];
            let (ask_price, ask_volume) = asks_vec[i];

            let line = format!(
                "{:<15} {:<15} {:<15} {:<15}\n",
                bid_price,
                bid_volume,
                ask_price,
                ask_volume
            );
            // Format and print the row
            out.push_str(&line);
        }
        out
        // pub fn best_quotes(&self) -> Option<(PriceLevel, PriceLevel)> {
        //     let best_ask = self.best_ask();
        //     let best_bid = self.best_bid();
        //     match (best_bid, best_ask) {
        //         (Some(a), Some(b)) => Some((a, b)),
        //         _ => None,
        //     }
        // }
    }
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


#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[tokio::test]
    async fn test_zero_quantity_updates() {
        let mut order_book = OrderBook::new();
        let updates = vec![
            PriceLevel {
                price: dec!(100),
                quantity: dec!(10),
                quote_type: QuoteType::ASK,
            },
            PriceLevel {
                price: dec!(100),
                quantity: Decimal::ZERO,
                quote_type: QuoteType::ASK,
            },
        ];

        order_book.update(&OrderBookUpdate::new(updates));

        assert!(order_book.asks.is_empty(), "Asks should be empty after a zero-quantity update");
    }

    #[tokio::test]
    async fn test_out_of_order_updates() {
        let mut order_book = OrderBook::new();
        let updates = vec![
            PriceLevel {
                price: dec!(101),
                quantity: dec!(5),
                quote_type: QuoteType::BID,
            },
            PriceLevel {
                price: dec!(100),
                quantity: dec!(10),
                quote_type: QuoteType::BID,
            },
        ];

        order_book.update(&OrderBookUpdate::new(updates));

        let first_bid = order_book.bids.iter().next_back().unwrap(); // Bids are in ascending order, so the last one is the highest
        assert_eq!(first_bid.0, &dec!(101), "The higher bid should come last in the BTreeMap");
        assert_eq!(first_bid.1, &dec!(5), "The quantity of the highest bid should be 5");
    }
}
