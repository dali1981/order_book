
// OrderBook
use kraken_ws_client::api::BookEvent;
use BookEvent as KrakenBookEvent;
use kraken_ws_client::types;
use rust_decimal::Decimal;
use crate::models::order_book::{PriceLevel, QuoteType};

use crate::models::order_book::OrderBookUpdate;
pub fn from_kraken(book_event: &KrakenBookEvent) -> OrderBookUpdate {
    let data = &book_event.data;
    let mut updates: Vec<PriceLevel> = Vec::new();

    data.iter().for_each(|data| {
        data.asks.iter().for_each(|level_data| {
            let price_level =
                PriceLevel::new(
                    Decimal::from_f64_retain(level_data.price).unwrap(),
                    Decimal::from_f64_retain(level_data.qty).unwrap(),
                    QuoteType::ASK
                );
            updates.push(price_level);
        });
        data.bids.iter().for_each(|level_data| {
            let price_level =
                PriceLevel::new(
                    Decimal::from_f64_retain(level_data.price).unwrap(),
                    Decimal::from_f64_retain(level_data.qty).unwrap(),
                    QuoteType::BID);
            updates.push(price_level);
        });
    }
    );


    data.iter().for_each(|book_data| {

    }
    );

    OrderBookUpdate::new(updates)
}