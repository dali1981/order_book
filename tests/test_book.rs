use websocket::models::kraken::book::{OrderBook, PriceLevel};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_orderbook() {
        let book = OrderBook::new("book-10".to_string(), "XBT/USD".to_string());
        assert_eq!(book.bids.len(), 0);
        assert_eq!(book.asks.len(), 0);
        assert_eq!(book.channel_name, "book-10");
        assert_eq!(book.pair, "XBT/USD");
    }

    #[test]
    fn test_update_orderbook_add_levels() {
        let mut book = OrderBook::new("book-10".to_string(), "XBT/USD".to_string());

        let price_level_1 = PriceLevel {
            price: "10000".to_string(),
            volume: "1.0".to_string(),
            timestamp: "1234567890.0" .to_string()
        };

        let price_level_2 = PriceLevel {
            price: "20000".to_string(),
            volume: "2.0".to_string(),
            timestamp: "1234567890.0" .to_string()
        };


        // Assuming update_level takes quantized price as i64, volume as f64, and timestamp as f64.
        book.update_level( price_level_1, true); // Add a bid
        book.update_level( price_level_2, false); // Add an ask

        assert_eq!(book.bids.len(), 1);
        assert!(book.bids.contains_key("10000.00"));
        assert_eq!(book.asks.len(), 1);
        assert!(book.asks.contains_key("20000.00"));
    }

    #[test]
    fn test_update_orderbook_remove_level() {
        let mut book = OrderBook::new("book-10".to_string(), "XBT/USD".to_string());

        let price_level_1 = PriceLevel {
            price: "10000".to_string(),
            volume: "1.0".to_string(),
            timestamp: "1234567890.0" .to_string()
        };

        let price_level_2 = PriceLevel {
            price: "10000".to_string(),
            volume: "0.0".to_string(),
            timestamp: "1234567890.0" .to_string()
        };
        // Add and then remove a bid by setting its volume to 0
        book.update_level(price_level_1, true);
        book.update_level(price_level_2, true); // Remove the bid

        assert_eq!(book.bids.len(), 0);
    }

    // Additional tests can include testing for:
    // - Multiple updates to the same price level
    // - Handling of invalid data (e.g., negative volume)
    // - Correct ordering of bids and asks if using a sorted data structure
    // - Performance benchmarks for large numbers of updates
}