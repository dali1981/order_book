use std::collections::HashMap;
use std::fmt;
use num_traits::{FromPrimitive, ToPrimitive};
use std::ops::Index;
use std::vec::Vec;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::models::types::*;

struct Order {
    qty: Qty,
    level_id: LevelIdx
}


#[derive(Debug)]
pub struct Pool<T: Default> {
    allocated: Vec<T>,
    free: Vec<usize>,
}

impl<T: Default> Pool<T> {
    pub fn new() -> Self {
        Pool {
            allocated: Vec::with_capacity(16),
            free: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Pool {
            allocated: Vec::with_capacity(capacity),
            free: Vec::new(),
        }
    }

    pub fn get(&mut self, idx: usize) -> &mut T {
        let size = self.allocated.len();
        assert!(idx < size, "Index out of bounds");
        &mut self.allocated[idx]
    }

    pub fn alloc(&mut self) -> usize {
        if self.free.is_empty() {
            let idx = self.allocated.len();
            self.allocated.push(T::default());
            idx
        } else {
            let idx = self.free.pop().unwrap();
            idx
        }
    }

    pub fn free(&mut self, idx: usize) -> () {
        self.free.remove(idx);
    }

}
impl<T : Default> Index<usize> for Pool<T> {
    type Output = T;
    fn index(&self, idx: usize) -> &T {
        & self.allocated[idx]
    }
}
#[derive(Debug)]
struct OidMap<T : Default>{
    data: Vec<T>,
    index: Vec<usize>,
}

impl<T: Default> OidMap<T> {
    pub fn new() -> Self {
        OidMap { data: Vec::<T>::new() , index: Vec::new()}
    }

    // Ensure there's enough space to insert an item at the given index (oid).
    // This method is analogous to the C++ reserve method but doesn't need to be called explicitly
    // before using `insert` because Vec::resize_with handles it internally.
    pub fn insert(&mut self, oid: usize, value: T) {
        if oid >= self.data.len() {
            // Resize the vector with None to ensure it has enough space to insert the new value
            self.data.resize_with(oid + 1, Default::default);
        }
        self.data[oid] = value;
    }

    // Retrieve a reference to the value at the given index (oid), if it exists.
    pub fn get(&self, oid: usize) -> &T {
        &self.data[oid]
    }

    // Optionally, implement removal if needed
    pub fn remove(&mut self, oid: usize) {
        self.data.remove(oid);

    }
}
#[derive(Debug)]
pub struct OrderBook {
    num_levels: u32,
    oid_map: OidMap<Level>,  // this determines the levels somehow?
    // sorted_levels: Vec<PriceLevel>,
    bids: SortedLevels,
    asks: SortedLevels,
    levels: Pool<Level>
}

impl OrderBook {

    pub fn new() -> Self {
        OrderBook{
            num_levels: 5,
            oid_map: OidMap::new(),
            bids: Vec::new(),
            asks: Vec::new(),
            levels: Pool::<Level>::new()
        }
    }

    fn process_entries<K, V>(&mut self, entries: &HashMap<K, V>, side: BuySell) -> ()
        where
            K: ToPrimitive + std::fmt::Debug + Copy + Clone,
            V: ToPrimitive + std::fmt::Debug + Copy + Clone,
    {
        for (&price, &qty) in entries {
            self.update_level(
                &Level::from_tuple((price.clone(), qty.clone())),
                side,
            );
        }
    }

    pub fn from_map<K, V>(_bids: HashMap<K, V>, _asks: HashMap<K, V>) -> Self
        where
            K: ToPrimitive + std::fmt::Debug + Copy + Clone,
            V: ToPrimitive + std::fmt::Debug + Copy + Clone,
    {
        let mut order_book = OrderBook::new();

        order_book.process_entries(&_bids, BuySell::Buy);
        order_book.process_entries(&_asks, BuySell::Sell);

        order_book
    }

    pub fn get_level(&self, idx: usize, buy_sell: BuySell) -> Level {
        let side = match buy_sell   {
            BuySell::Buy => &self.bids,
            BuySell::Sell => &self.asks,
        };

         match side.get(idx) {
            Some(p) => {
                let i = p.level_idx().value();
                self.levels[i].clone()
            },
            None => Level::default(),
        }
    }

    fn get_qty(&self, price_level: &PriceLevel) -> Qty {
        let level_idx = price_level.level_idx().value();
        let level = &self.levels[level_idx];
        level.qty
    }

    fn cross(level: &Level, buy_sell: BuySell, opposite_side: &PriceLevel) -> bool {
        match buy_sell {
            BuySell::Buy => level.price > opposite_side.price(),
            BuySell::Sell => level.price < opposite_side.price(),
        }
    }

    fn cross_(&mut self, level: &Level, buy_sell: BuySell) -> bool {
         match  buy_sell {
            BuySell::Buy => {
                match self.asks.get(0) {
                    Some(ask) => level.price >= ask.price(),
                    None => false,
                }
            }
            BuySell::Sell => match self.bids.get(0) {
                    Some(bid) => level.price <= bid.price(),
                    None => false,
                }
        }
    }
    //
    // fn get_level(&self, price_level: &PriceLevel) -> &mut Level {
    //     let level_idx = price_level.level_idx();
    //     self.levels.get(level_idx.value())
    // }

    fn remove_price_level(&mut self, idx: usize, buy_sell: BuySell) {
        let price_level = match buy_sell {
            BuySell::Buy => self.bids.remove(idx),
            BuySell::Sell => self.asks.remove(idx)
        };
        let level_idx = price_level.level_idx();
        self.levels.free.push(level_idx.value());
    }

    fn get_best_price_level(&self, side: BuySell) -> &PriceLevel {
        match side {
            BuySell::Buy => self.bids.iter().next_back().unwrap(),
            BuySell::Sell => self.asks.iter().next().unwrap(),
        }
    }

    // fn execute(
    //     &mut self,
    //     level: &Level,
    //     buy_sell: &BuySell,
    //     opposite_side: &mut PriceLevel,
    // ) -> Level {
    //     let mut remaining_qty = level.qty.value();
    //     let mut remaining_price = level.price.value();
    //
    //     // Execute and update remaining quantities until full or exhausted
    //     while_both_positive(&mut remaining_qty, &mut opposite_side.quantity.0, |order_qty, book_qty| {
    //         let execution_qty = std::cmp::min(*order_qty, *book_qty);
    //         *order_qty -= execution_qty;
    //         *book_qty -= execution_qty;
    //
    //         // Update remaining price if opposite side is exhausted
    //         if *book_qty == 0 {
    //             remaining_price = opposite_side.price().value();
    //             self.remove_price_level(opposite_side.level_idx().0);
    //             return false; // Stop iterating
    //         }
    //
    //         // Check for potential price level update
    //         if self.get_price_level_idx(remaining_qty, buy_sell) != level.level_idx.0 {
    //             self.update_price_level(*level, level.level_idx.0, self.get_price_level_idx(remaining_qty, buy_sell));
    //             return false; // Stop iterating
    //         }
    //
    //         true // Continue iterating
    //     });
    //
    //     // Return the remaining order
    //     Level::new(Price(remaining_price), Qty(remaining_qty))
    // }

    fn has_better_price(price_level: &PriceLevel, new_level: &Level) -> bool {
        match price_level.side() {
            BuySell::Buy => price_level.price() < new_level.price(),
            BuySell::Sell => price_level.price() > new_level.price(),
        }
    }

    fn update_level(&mut self, level: &Level, buy_sell: BuySell) {
        if self.asks.is_empty() && matches!(buy_sell, BuySell::Sell) {
            let level_idx = self.levels.alloc();
            // self.levels.get(level_idx).clone_from(&level); // Efficiently copy level data
            let level_ = self.levels.get(level_idx);
            level_.price = level.price;
            level_.qty = level.qty;
            let price_level = PriceLevel::new(level.price, LevelIdx(level_idx), buy_sell);
            self.asks.insert(0, price_level);
            return;
        }

        if self.bids.is_empty() && matches!(buy_sell, BuySell::Buy) {
            let level_idx = self.levels.alloc();
            // self.levels.get(level_idx).clone_from(&level); // Efficiently copy level data
            let level_ = self.levels.get(level_idx);
            level_.price = level.price;
            level_.qty = level.qty;
            let price_level = PriceLevel::new(level.price, LevelIdx(level_idx), buy_sell);
            self.bids.insert(0, price_level);
            return;
        }


        {
            if self.cross_(&level, buy_sell) {
                // Improve or execute against the other side
                self.remove_price_level(0, buy_sell.other_side());
                // let remaining = self.execute(&level, buy_sell, opposite_side);
                self.update_level(level, buy_sell);
                return;
            }
        }

        let sorted_levels = match buy_sell {
            BuySell::Buy =>  &mut self.bids,
            BuySell::Sell => &mut self.asks,
        };

        let mut found = false;
        let mut insert_index = 0;

        for price_level in sorted_levels.iter() {

            if price_level.price() == level.price() {
                // Insert before the current price level as it's a better price for the buy order.
                found = true;
                break;
            } else if OrderBook::has_better_price(&price_level, &level) {
                // No need to check further as the order book is sorted by price (descending for bids, ascending for asks).
                break;
            }
            insert_index += 1;
        }

        if !found {
            // New price level
            let level_idx = self.levels.alloc();
            // self.levels.get(level_idx).clone_from(&level); // Efficiently copy level data
            let level_ = self.levels.get(level_idx);
            level_.price = level.price;
            level_.qty = level.qty;

            let new_price_level = PriceLevel::new(level.price, LevelIdx(level_idx), buy_sell.clone());
            sorted_levels.insert(insert_index, new_price_level);
        } else {
            // Update existing level quantity

            let level_idx = sorted_levels[insert_index].level_idx();
            let level_ = self.levels.get(level_idx.value());
            level_.price = level.price;
            level_.qty = level.qty;
        }
    }

    pub fn get_repr(&self) -> (HashMap<Price, Qty>, HashMap<Price, Qty>) {
        let mut bids = HashMap::<Price, Qty>::new();
        let mut asks = HashMap::<Price, Qty>::new();

        for bid_price_level in self.bids.iter() {
            bids.insert(bid_price_level.price(), self.get_qty(bid_price_level));
        }
        for ask_price_level in self.asks.iter() {
            asks.insert(ask_price_level.price(), self.get_qty(ask_price_level));
        }
        (bids, asks)
    }
}

impl fmt::Display for OrderBook {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", "Order Book Table");
        writeln!(f, "{:10} {:10} | {:>10} {:>10}", "Bid", "Ask", "Qty", "Price");
        // writeln!(f, "{:-:10} {:-:10} | {:-:10} {:-:10}", "", "", "", "");


        // Print each level in the order book
        let mut num_levels = 0;
        for (bid_price_level, ask_price_level) in self.bids.iter().zip(self.asks.iter()) {
            writeln!(f,
                     "{:10} {:10} | {:>10} {:>10.2}",
                     self.get_qty(bid_price_level),
                     bid_price_level.price(),
                     ask_price_level.price(),
                     self.get_qty(ask_price_level)
            )?;
            num_levels += 1;
        }

        if num_levels == 0 {
            writeln!(f, "No orders in the order book");
        }

        Ok(())
    }
}

// fn while_both_positive<F>(
//     qty1: &mut Decimal,
//     qty2: &mut Decimal,
//     mut f: F,
// ) -> bool
//     where
//         F: FnMut(&mut u64, &mut u64) -> bool,
// {
//     while *qty1 > 0 && *qty2 > 0 {
//         let execution_qty = std::cmp::min(*qty1, *qty2);
//         f(qty1, qty2);
//         *qty1 -= execution_qty;
//         *qty2 -= execution_qty;
//
//         // Early return if the closure indicates stopping
//         if !f(qty1, qty2) {
//             return false;
//         }
//     }
//
//     true
// }

pub struct OrderBookUpdate {
    pub level: Level,
    pub buy_sell: BuySell
}

impl OrderBookUpdate {
    pub fn new(price: Decimal, qty: Decimal, buy_sell: BuySell) -> Self {
        OrderBookUpdate{
            level: Level::new(Price(price), Qty(qty)),
            buy_sell
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use crate::models::types::*;

    #[tokio::test]
    async fn test_zero_quantity_updates() {
        let mut order_book = OrderBook::new();
        let updates = vec![
            OrderBookUpdate::new(dec!(100), dec!(10), BuySell::Sell),
            OrderBookUpdate::new(dec!(99), dec!(5), BuySell::Buy)
            ];

        updates.iter().for_each(|update| {
            order_book.update_level(&update.level, update.buy_sell)
        });

        assert_eq!(order_book.asks[0].price().value(), dec!(100), "Asks should be (100, 10)");
        // assert_eq!(order_book.asks[0].qty().value(), dec!(10), "Asks should be (100, 10)");
        assert_eq!(order_book.bids[0].price().value(), dec!(99), "Bids should be (100, 10)");
        // assert_eq!(order_book.bids[0].qty().value(), dec!(5), "Bids should be (100, 10)");
    }

    #[test]
    fn test_out_of_order_updates() {
        let bids = vec![(100, 5), (101, 5)];

        let expected_b = Level::from_tuple((101, 5));


        let mut order_book = OrderBook::new();

        for bid in bids {
            let level = Level::from_tuple(bid);
            order_book.update_level(&level, BuySell::Buy);
        }
        let level_b = order_book.get_level(0, BuySell::Buy);

        assert_eq!(level_b, expected_b, "Unordered bid updates not updated correctly");
    }

    #[test]
    fn test_in_order_updates() {
        let bids = vec![(101, 5), (100, 5)];

        let expected_b = Level::from_tuple((101, 5));


        let mut order_book = OrderBook::new();

        for bid in bids {
            let level = Level::from_tuple(bid);
            order_book.update_level(&level, BuySell::Buy);
        }
        let level_b = order_book.get_level(0, BuySell::Buy);

        assert_eq!(level_b, expected_b, "ordered bid levels not updated correctly");
    }

    #[test]
    fn test_multiple_levels() {
        let mut order_book = OrderBook::new();

        // Add initial levels
        order_book.update_level(&Level::new(Price(dec!(101)), Qty(dec!(11))), BuySell::Sell);
        order_book.update_level(&Level::new(Price(dec!(100)), Qty(dec!(10))), BuySell::Sell);

        order_book.update_level(&Level::new(Price(dec!(99)), Qty(dec!(9))), BuySell::Buy);
        order_book.update_level(&Level::new(Price(dec!(98)), Qty(dec!(8))), BuySell::Buy);

        let (bids, asks) = order_book.get_repr();

        assert_eq!(asks[&Price(dec!(101))], Qty(dec!(11)), "error");
        assert_eq!(asks[&Price(dec!(100))], Qty(dec!(10)), "error");
        assert_eq!(bids[&Price(dec!(99))], Qty(dec!(9)), "error");
        assert_eq!(bids[&Price(dec!(98))], Qty(dec!(8)), "error");
        // Crossing order
        let crossing_update = OrderBookUpdate::new(dec!(99), dec!(5), BuySell::Sell);
        let is_cross = OrderBook::cross(
            &crossing_update.level, crossing_update.buy_sell, &order_book.get_best_price_level(BuySell::Sell)
        );
        assert!(is_cross, "Order should cross");
        // order_book.update_level(&crossing_update.level, "Order should cross");

        // Assuming `update_level` handles crossing by removing or updating opposite side levels
        // assert!(OrderBook::cross(&crossing_update.level, crossing_update.buy_sell, &order_book.get_best_price_level(BuySell::Sell)), "Order should cross");

        // Add assertions based on how your system handles crosses, e.g., removing levels, updating quantities, etc.
    }


    #[test]
    fn test_cross() {
        let asks = HashMap::from([
            (101, 10),
            (102, 25),
            (108, 22)
        ]);

        let bids = HashMap::from([
            (100, 90),
            (99, 5),
            (98, 1000)
        ]);

        let mut order_book = OrderBook::from_map(bids, asks);

        // Crossing order
        let crossing_update =
            OrderBookUpdate::new(dec!(100), dec!(5), BuySell::Sell);

        order_book.update_level(&crossing_update.level, crossing_update.buy_sell);

        let level_a = order_book.get_level(0, BuySell::Sell);
        let level_b = order_book.get_level(0, BuySell::Buy);
        let expected_b = Level::new(Price(dec!(99)), Qty(dec!(5)));
        let expected_a = Level::new(Price(dec!(100)), Qty(dec!(5)));


        assert_eq!(level_a, expected_a, "Order should cross");
        assert_eq!(level_b, expected_b, "Order should cross");
        // order_book.update_level(&crossing_update.level, "Order should cross");

        // Assuming `update_level` handles crossing by removing or updating opposite side levels
        // assert!(OrderBook::cross(&crossing_update.level, crossing_update.buy_sell, &order_book.get_best_price_level(BuySell::Sell)), "Order should cross");

        // Add assertions based on how your system handles crosses, e.g., removing levels, updating quantities, etc.
    }



    // #[tokio::test]
    // async fn test_order_crossing() {
    //     let mut order_book = OrderBook::new();
    //
    //     // Add initial levels
    //     order_book.update_level(&Level::new(Price(dec!(100)), Qty(dec!(10))), BuySell::Sell);
    //     order_book.update_level(&Level::new(Price(dec!(98)), Qty(dec!(10))), BuySell::Buy);
    //
    //     // Crossing order
    //     let crossing_update = OrderBookUpdate::new(dec!(99), dec!(5), BuySell::Buy);
    //     order_book.update_level(&crossing_update.level, crossing_update.buy_sell);
    //
    //     // Assuming `update_level` handles crossing by removing or updating opposite side levels
    //     assert!(order_book.cross(&crossing_update.level, crossing_update.buy_sell, &order_book.get_best_price_level(BuySell::Sell)), "Order should cross");
    //
    //     // Add assertions based on how your system handles crosses, e.g., removing levels, updating quantities, etc.
    // }

}
