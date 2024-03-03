use std::cmp::Ordering;

use std::fmt;
use num_traits::{FromPrimitive, ToPrimitive};

use rust_decimal::Decimal;
use crate::models::order_book_2::OrderBook;


#[derive(Debug, PartialEq, Eq, PartialOrd, Hash, Copy, Clone)]
pub struct Price(pub Decimal);

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
pub struct  Qty(pub Decimal);
#[derive(Debug, Copy, Clone)]
pub struct LevelIdx(pub usize);
pub type SortedLevels = Vec<PriceLevel>;

#[derive(Debug, Copy, Clone)]
pub enum BuySell {
    Buy,
    Sell
}


// Improved Price implementation:
impl Price {
    // Access the underlying Decimal directly for convenience
    pub fn value(&self) -> Decimal {
        self.0
    }

    // Methods for common operations with Decimal
    pub fn add(&self, other: Decimal) -> Price {
        Price(self.0 + other)
    }

    pub fn subtract(&self, other: Decimal) -> Price {
        Price(self.0 - other)
    }

    // ... other useful methods as needed
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Qty {
    // Access the underlying Decimal directly for convenience
    pub fn value(&self) -> Decimal {
        self.0
    }

    // Methods for common operations with Decimal
    pub fn add(&self, other: Decimal) -> Price {
        Price(self.0 + other)
    }

    pub fn subtract(&self, other: Decimal) -> Price {
        Price(self.0 - other)
    }

    // ... other useful methods as needed
}

impl fmt::Display for Qty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


impl LevelIdx {
    // Access the underlying Decimal directly for convenience
    pub fn value(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Level {
    pub price: Price,
    pub qty: Qty
}

impl PartialEq for Level {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price && self.qty == other.qty
    }
}

impl Eq for Level {}




impl Default for Level {
    fn default() -> Self {
        Level::new(Price(Decimal::ZERO), Qty(Decimal::ZERO))
    }
}

impl Level {
    pub fn new(price: Price, qty: Qty) -> Self {
        Level{price, qty}
    }

    pub fn from_tuple<P, Q>(tuple: (P, Q)) -> Self
        where
            P: ToPrimitive + std::fmt::Debug,
            Q: ToPrimitive + std::fmt::Debug,
    {
        let mut order_book = OrderBook::new();

        let price_decimal = Decimal::from_f64(tuple.0.to_f64().unwrap_or_default()).unwrap_or_default();
        let qty_decimal = Decimal::from_f64(tuple.1.to_f64().unwrap_or_default()).unwrap_or_default();

        Level::new(Price(price_decimal), Qty(qty_decimal))
    }

    // pub fn from_dec(price: Decimal, qty: Decimal) -> Self {
    //     Level::new(Price(price), Qty(qty))
    // }

    pub fn price(&self) -> Price {
        self.price
    }
    pub fn reduce_qty(&self, qty: &Qty) -> Level {
        Level::new((&self).price, Qty(self.qty.0 - qty.0))
    }

    pub fn set_qty(&mut self, qty: Qty) -> () {
        &Level::new(Price(self.price.0), qty);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PriceLevel {
    price: Price,
    level_idx: LevelIdx,
    side: BuySell
}


impl PriceLevel {
    pub fn new(price: Price, level_idx: LevelIdx, side: BuySell) -> PriceLevel {
        PriceLevel{price, level_idx, side}
    }

    pub fn price(&self) -> Price {
        self.price
    }

    pub fn side(&self) -> BuySell {
        self.side
    }

    pub fn level_idx(&self) -> LevelIdx {
        self.level_idx
    }
}

impl PartialOrd for PriceLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for PriceLevel {}
impl Ord for PriceLevel {
    fn cmp(&self, other: &Self) -> Ordering {
        self.price.partial_cmp(&other.price).unwrap()
    }
}
impl PartialEq for PriceLevel {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price
    }
}
// impl PartialOrd for PriceLevel{
//     fn ge(&self, other: &Self) -> bool {
//         match &self.side {
//             ASK => self.price >= other.price,
//             BID => self.price <= other.price,
//         }
//     }
//     fn gt(&self, other: &Self) -> bool {
//         match &self.side {
//             ASK => self.price > other.price,
//             BID => self.price < other.price,
//         }
//     }
//     fn le(&self, other: &Self) -> bool {
//         match &self.side {
//             ASK => self.price <= other.price,
//             BID => self.price >= other.price,
//         }
//     }
//     fn lt(&self, other: &Self) -> bool {
//         match &self.side {
//             ASK => self.price < other.price,
//             BID => self.price > other.price,
//         }
//     }
//
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         todo!()
//     }
// }
//
// impl PartialOrd for Price {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         Some(self.0.cmp(&other.0))
//     }
// }
//
// impl Ord for Price {
//     fn cmp(&self, other: &Self) -> Ordering {
//         self.partial_cmp(other).unwrap()
//     }
// }
//
// impl Deref for Price {
//     type Target = Decimal;
//
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }
//



impl BuySell {
    pub fn other_side(self) -> Self {
        match self {
            BuySell::Buy => BuySell::Sell,
            BuySell::Sell => BuySell::Buy,
        }
    }
}


impl fmt::Display for BuySell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuySell::Buy => write!(f, "BID"),
            BuySell::Sell => write!(f, "ASK"),
        }
    }
}