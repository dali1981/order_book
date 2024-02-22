

use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};
use std::{fmt, result};
use serde_json::Result;

use crate::messages::IncomingMsg;

use crate::models::kraken::book::parse::parse_message;


#[derive(Debug)]
struct ParseError(String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ParseError {}


use serde_json::{Error as SerdeError, Value};
#[derive(Debug)]
pub enum MyError {
    ParseError(String),
    SerdeError(SerdeError),
}



impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyError::ParseError(e) => write!(f, "Parse error: {}", e),
            MyError::SerdeError(e) => write!(f, "Serde JSON error: {}", e),
        }
    }
}

// // Implement From for ParseError and SerdeError to convert them into MyError
// impl From<ParseError> for MyError {
//     fn from(error: ParseError) -> Self {
//         MyError::ParseError(error)
//     }
// }

impl From<SerdeError> for MyError {
    fn from(error: SerdeError) -> Self {
        MyError::SerdeError(error)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: String,
    pub volume: String,
    pub timestamp: String,
}

impl fmt::Display for PriceLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Price: {}, Volume: {}, Timestamp: {}", self.price, self.volume, self.timestamp)
    }
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookUpdate {
    #[serde(rename = "a")]
    asks: Option<Vec<PriceLevel>>,

    #[serde(rename = "b")]
    bids: Option<Vec<PriceLevel>>,

    c: Option<String>, // Assuming 'c' is present in updates for checksum.
}


#[derive(Debug, Clone)]
pub struct OrderBook {
    pub bids: HashMap<String, PriceLevel>,
    pub asks: HashMap<String, PriceLevel>,
}

impl fmt::Display for OrderBook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "OrderBook ",)?;
        writeln!(f, "Bids: ")?;
        for (key, bid) in &self.bids {
            write!(f, "{}: {} ", key, bid)?;
        }
        writeln!(f, "Asks: ")?;
        for (key, ask) in &self.asks {
            write!(f, "{}: {} ", key, ask)?;
        }
        Ok(())
    }
}


impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            bids: HashMap::new(),
            asks: HashMap::new(),
        }
    }

    pub fn from_snapshot(snapshot: String){
        let initial_update = snapshot.replace("bs", "b");
        let order_book_update = parse_message(&initial_update).unwrap();
        OrderBook::init(order_book_update);
    }

    pub fn init(update: OrderBookUpdate) -> result::Result<OrderBook, MyError>{
        let mut initialized = OrderBook::new();

        match update.asks {
            Some(asks) => {
                asks.iter().for_each(|ask| {
                    initialized.update_level(ask.clone(), false);
                });
            },
            None => return Err(MyError::ParseError("MissingAsks".to_string())),
        }

        match update.bids {
            Some(bids) => {
                bids.iter().for_each(|bid| {
                    initialized.update_level(bid.clone(), true);
                });
            },
            None => return Err(MyError::ParseError("MissingBids".to_string())),
        }

        Ok(initialized)
    }



    fn update_from_message(&mut self, message: String) {
        let update: serde_json::Result<OrderBookUpdate> = serde_json::from_str(&message);

        match update {
            Ok(update) => {
                if let Some(asks) = update.asks {
                    for ask in asks {
                        self.update_level(ask, false);
                    }
                }
                if let Some(bids) = update.bids {
                    for bid in bids {
                        self.update_level(bid, true);
                    }
                }
            },
            Err(e) => println!("Error parsing update message: {}", e),
        }
    }


    pub fn update_level(&mut self, level: PriceLevel, is_bid: bool) {
        let price = level.price.parse::<f64>().expect("Error parsing price to f64");
        let volume = level.volume.parse::<f64>().expect("Error parsing volume to f64");
        let timestamp = level.timestamp.parse::<f64>().expect("Error parsing timestamp to f64");

        let level_key = format!("{:.10}", price);

        if is_bid {
            if volume == 0.0 {
                self.bids.remove(&level_key);
            } else {
                self.bids.insert(level_key, level);
            }
        } else {
            if volume == 0.0 {
                self.asks.remove(&level_key);
            } else {
                self.asks.insert(level_key, level);
            }
        }
    }

}

pub mod parse {
    use super::*;
    use regex::Regex;

    use std::error::Error;
    use std::result;
    use futures::TryFutureExt;




    pub fn parse_message(msg: &String) -> result::Result<OrderBookUpdate, MyError> {
        let re = Regex::new(r#"\[\s*(\d+)\s*,\s*(\{.*?\})\s*,\s*"(.*?)"\s*,\s*"(.*?)"\s*\]"#).unwrap();

        match re.captures(msg) {
            Some(caps) => {
                let channel_id = &caps[1];
                let payload = &caps[2];
                let channel_name = &caps[3];
                let pair = &caps[4];
                println!("payload: {}", payload);

                let parsed_json: Value = serde_json::from_str(payload)?;

                if let Some((first_key, _first_value)) = parsed_json.as_object().and_then(|obj| obj.iter().next()) {
                    if first_key == "as" {   // initial orderbook
                        let initial_update = payload.replace("bs", "b");
                        serde_json::from_str::<OrderBookUpdate>(&initial_update).map_err(MyError::from)

                    } else if first_key == "a" {
                        serde_json::from_str::<OrderBookUpdate>(payload).map_err(MyError::from)
                    } else {  Err(MyError::ParseError("cant parse".to_string()))  }
                } else {  Err(MyError::ParseError("cant parse".to_string()))  }


            },
            None => {
                let error_msg = "Message format does not match expected pattern".to_string();
                Err(MyError::ParseError(error_msg))
            }
        }

    }


    ///
    ///
    /// # Arguments
    ///
    /// * `json_str`:
    ///
    /// returns: Result<<unknown>, Error>
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    fn parse_json(json_str: &str) -> Result<OrderBookUpdate> {
        serde_json::from_str::<OrderBookUpdate>(json_str)
    }
}

// #[derive(Debug, Clone)]
// struct IncomingMsg {
//     as: Option<Vec<Vec<String>>>,
//     bs: Option<Vec<Vec<String>>>,
//     // Additional fields as needed
// }
//
// // Example usage
// fn main() {
//     let mut book = OrderBook::new("book-10".to_string(), "XBT/USD".to_string());
//
//     // Example message update
//     let msg = IncomingMsg {
//         as: Some(vec![vec!["51612.20000".to_string(), "5.51312925".to_string(), "1708447713.035616".to_string()]]),
//         bs: None,
//         // fill in other fields as needed
//     };
//
//     book.update_from_message(msg);
//     println!("{:?}", book);
// }



// Snapshot payload
//
// Name	Type	  Description
// channelID	  integer	Channel ID of subscription - deprecated, use channelName and pair
// (Anonymous)    object
// as	array	  Array of price levels, ascending from best ask
// Array	array	Anonymous array of level values
// price	decimal	Price level
// volume	decimal	Price level volume, for updates volume = 0 for level removal/deletion
// timestamp	decimal	Price level last updated, seconds since epoch
// bs	array	Array of price levels, descending from best bid
// Array	array	Anonymous array of level values
// price	decimal	Price level
// volume	decimal	Price level volume, for updates volume = 0 for level removal/deletion
// timestamp	decimal	Price level last updated, seconds since epoch
// channelName	string	Channel Name of subscription
// pair	string	Asset pair
// Example of snapshot payload
//
// [
//   0,
//   {
//     "as": [
//       [
//         "5541.30000",
//         "2.50700000",
//         "1534614248.123678"
//       ],
//       [
//         "5541.80000",
//         "0.33000000",
//         "1534614098.345543"
//       ],
//       [
//         "5542.70000",
//         "0.64700000",
//         "1534614244.654432"
//       ]
//     ],
//     "bs": [
//       [
//         "5541.20000",
//         "1.52900000",
//         "1534614248.765567"
//       ],
//       [
//         "5539.90000",
//         "0.30000000",
//         "1534614241.769870"
//       ],
//       [
//         "5539.50000",
//         "5.00000000",
//         "1534613831.243486"
//       ]
//     ]
//   },
//   "book-100",
//   "XBT/USD"
// ]