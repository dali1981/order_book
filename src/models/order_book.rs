use std::collections::BTreeMap;
use std::fmt;
use kraken_ws_client::api::{BookData};
use rust_decimal::Decimal;
use std::string::String;
use crate::models::order_book::QuoteType::{ASK, BID};

type Bids = BTreeMap<Price, Qty>;
type Asks = BTreeMap<Price, Qty>;

type Price = Decimal;
type Qty = Decimal;


#[derive(Debug)]
pub struct OrderBook {
    bids: Bids,
    asks: Asks,
    is_empty: bool
}

impl Clone for OrderBook {
    fn clone(&self) -> Self {
        OrderBook {
            bids: self.bids.clone(),
            asks:self.asks.clone(),
            is_empty: self.is_empty
        }
    }
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            is_empty: true
        }
    }


    fn insert_order(&mut self, price: Price, quantity: Qty, quote_type: QuoteType) {
        match quote_type {
            BID => {
                if quantity == Decimal::ZERO {
                    // If the quantity is zero, remove the price level from the bids
                    self.bids.remove(&price);
                } else {
                    // Otherwise, insert or update the price level with the new quantity
                    self.bids.insert(price, quantity);
                }
            },
            ASK => if quantity == Decimal::ZERO {
                // If the quantity is zero, remove the price level from the bids
                self.asks.remove(&price);
            } else {
                // Otherwise, insert or update the price level with the new quantity
                self.asks.insert(price, quantity);
            },
        };
    }


    fn execute_sell_limit(&self, price: Price, qty: Qty) -> Vec<(Price, Qty)> {
        let mut sell_order_qty = qty;
        let mut modifications: Vec<(Price, Qty)> = Vec::new();

        for (&bid_price, &bid_qty) in self.bids.iter().rev() {
            if bid_price < price || sell_order_qty.is_zero() {
                // If the bid price is less than the sell price, or no quantity left to sell, exit loop
                break;
            }
            if bid_qty <= sell_order_qty {
                modifications.push((bid_price, Decimal::ZERO));
                sell_order_qty -= bid_qty; // Assuming qty is of type Decimal

            } else {
                let remaining_in_the_bid_level = bid_qty - sell_order_qty;
                modifications.push((bid_price, remaining_in_the_bid_level));

                sell_order_qty = Decimal::ZERO; // Update sell_order_qty accordingly
                break;

            }
        }
        modifications
    }

    fn execute_buy_limit( &mut self, price: Price, qty: Qty) -> Vec<(Price, Qty)>  {
        let mut buy_order_qty = qty;
        let mut modifications: Vec<(Price, Qty)> = Vec::new();
        // todo there must be a bug: if the limit is empty at some point, the remaining of the order will stay in the book
        // todo this scenario isnt cobered

        for (&ask_price, &ask_qty) in self.asks.iter() {
            if ask_price > price || buy_order_qty.is_zero() {
                // If the bid price is less than the sell price, or no quantity left to sell, exit loop
                break;
            }
            if ask_qty <= buy_order_qty {
                modifications.push((ask_price, Decimal::ZERO));
                buy_order_qty -= ask_qty; // Assuming qty is of type Decimal

            } else {
                let remaining_in_the_ask_level = ask_qty - buy_order_qty;
                modifications.push((ask_price, remaining_in_the_ask_level));

                buy_order_qty = Decimal::ZERO; // Update sell_order_qty accordingly
                break;

            }
        }

        modifications
    }

    pub fn update_from_kraken(&mut self, update: &Vec<BookData>) {
        if self.is_empty {
            for book_data in update.iter() {
                for level_data in book_data.asks.iter() {
                    self.insert_order(level_data.price, level_data.qty, ASK)
                }
                for level_data in book_data.bids.iter() {
                    self.insert_order(level_data.price, level_data.qty, BID)
                }
            }
            // construct the order book from the update => straightforward update
        } else {
            // for each limit update, loop over the book and decide if it leads to execution
            // if it leads to execution, proceed with the execution
            // -> loop over the opposite side and remove liquidity
            // if not proceed with a regular update
            for book_data in update.iter() {
                for level_data in book_data.asks.iter() {
                    let price_in = level_data.price;
                    let qty_in = level_data.qty;
                    match self.best_bid() {
                        Some((bid, qty)) => {
                            if bid >= price_in {
                                let modifications = self.execute_sell_limit(price_in, qty_in);
                                // Apply modifications
                                for (price, qty) in modifications {
                                    self.insert_order(price, qty, BID);
                                }
                            } else {
                                self.insert_order(price_in, qty, ASK);
                            }
                        },
                        None => (),
                    }
                }
                for level_data in book_data.asks.iter().rev() {
                    let price_in = level_data.price;
                    let qty_in = level_data.qty;

                    match self.best_ask() {
                        Some((ask, qty)) => {
                            if ask <= price_in {
                                let modifications = self.execute_buy_limit(price_in, qty_in);
                                // Apply modifications
                                for (price, qty) in modifications {
                                    self.insert_order(price, qty, ASK);
                                }
                            } else {
                                self.insert_order(price_in, qty, BID)
                            }
                        },
                        None => (),
                    }
                }
            }
        }
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

    pub fn update(&mut self, update: &Vec<PriceLevel>) {
        update.iter().for_each(|price_level| {
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

#[derive(Debug)]
pub enum QuoteType{
    BID, ASK
}

impl fmt::Display for QuoteType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuoteType::BID => write!(f, "BID"),
            QuoteType::ASK => write!(f, "ASK"),
        }
    }
}

#[derive(Debug)]
pub struct PriceLevel {
    price: Price,
    quantity: Qty,
    quote_type: QuoteType
}

impl PriceLevel {
    pub fn new(price: Price, quantity: Qty, quote_type: QuoteType) -> Self {
        PriceLevel{price, quantity, quote_type}
    }
}

#[derive(Debug)]
pub struct OrderBookUpdate {
    price_levels: Vec<PriceLevel>
}

impl OrderBookUpdate {
    pub fn new(price_levels: Vec<PriceLevel>) -> Self {
        OrderBookUpdate { price_levels }
    }

    pub fn log_msg(&self) -> String {
        let mut formatted = String::new();
        for level in &self.price_levels{
            let line = format!("{};{};{}\n", level.price, level.quantity, level.quote_type);
            formatted.push_str(&line);
        }
        formatted
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

        // order_book.update(&OrderBookUpdate::new(updates));

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

        // order_book.update(&OrderBookUpdate::new(updates));

        let first_bid = order_book.bids.iter().next_back().unwrap(); // Bids are in ascending order, so the last one is the highest
        assert_eq!(first_bid.0, &dec!(101), "The higher bid should come last in the BTreeMap");
        assert_eq!(first_bid.1, &dec!(5), "The quantity of the highest bid should be 5");
    }
}
