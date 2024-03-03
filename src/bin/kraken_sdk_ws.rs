use futures::StreamExt;
use kraken_ws_client::api::{SubscribeTickerRequest, TickerEvent, SubscribeBookRequest, BookEvent, BookData};
use kraken_ws_client::client::MyMessage;
use kraken_ws_client::types::Depth;
use serde::{Deserialize, Serialize};
use serde::de::Unexpected::Str;
use std::string::String;
use websocket::models::order_book::{OrderBook, OrderBookUpdate};


use tokio::sync::mpsc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::{debug, error};
// Provides async write functions for File
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber;
use tracing_subscriber::{fmt, EnvFilter};
use tracing_appender::rolling;
use tracing_subscriber::util::SubscriberInitExt;

use websocket::models::order_book::PriceLevel;

use uuid::Uuid;

// #[derive(Debug)]
// pub struct MyMessage {
//     correlation_id: Uuid,
//     payload: String,
// }

#[derive(Debug)]
pub struct DisplayMessage {
    correlation_id: Uuid,
    payload: (OrderBook, OrderBookUpdate),
}

use tracing_subscriber::layer::SubscriberExt;
use websocket::models::order_book::QuoteType::{ASK, BID};

fn setup_logging() {
    // File appender setup for logging to a file, with non-blocking behavior.
    let file_appender = rolling::never("./logs", "tracing_logs.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Environment filter setup. Defaults to "debug" level if RUST_LOG isn't explicitly set.
    let filter_layer = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"));


    // Formatting layer for console output, including timestamps and source file info.
    let fmt_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_timer(fmt::time::uptime())
        .pretty() // Use pretty printing; you can change to .compact() for less verbose output.
        // .with_file(true) // Include the name of the source file in the logs.
        .with_line_number(true); // Include the line number in the logs.

    // Formatting layer for file output, similar configuration as for console.
    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_timer(fmt::time::uptime())
        .pretty() // Adjust as needed, .json() is also available for structured logging.
        // .with_file(true)
        .with_line_number(true);

    // Combine everything into a subscriber and set it as the global default.
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(file_layer)
        .init();
}

#[tokio::main]
async fn main() {
    setup_logging();

    let mut client = kraken_ws_client::connect_public()
        .await
        .expect("cannot connect");


    let (data_tx, mut data_rx) = mpsc::channel::<MyMessage>(128); // Create a channel with a buffer size of 32.
    let (display_tx, mut display_rx) = mpsc::channel::<DisplayMessage>(128); // Create a channel with a buffer size of 32.

    // Spawn a task to handle printing.
    tokio::spawn(async move {
        // Open or create a file to write. `File::create` will truncate the file if it already exists.
        let mut file = File::create("order_book_log.txt").await.expect("Failed to create file");

        while let Some(message) = display_rx.recv().await {
            let (ob, ob_update) = message.payload;
            let correl_id = message.correlation_id;
            let quote = ob.best();

            // Write to stdout
            debug!("{}-{}", correl_id,  &quote);
            // debug!("{}", &quote);

            let mut  log = ob_update.log_msg();
            // log = format!("{};{}", correl_id.to_string(), log);
            //todo format the string in another class: we may need to add additional columns

            // Write to file, appending a newline character for each message
            // if let Err(e) = file.write_all(format!("{}\n", &quote).as_bytes()).await {
            if let Err(e) = file.write_all(log.as_bytes()).await {
                error!("Failed to write to file: {}", e);
                // Decide how to handle the write error. For example, you might want to break the loop,
                // or you might simply log the error and continue trying to write new messages.
            }
        }
    });

    // Spawn a task to handle printing.
    tokio::spawn(async move {
        let mut order_book:  OrderBook = OrderBook::new();

        while let Some(message) = data_rx.recv().await {
            // println!();
            // // Handle trade message
            // println!("Handling trade message: {}", message);
            let msg = message.payload();
            let correlation_id = message.correl_id();
            let parsed = serde_json::from_str::<BookEvent>(&msg).ok();
            match parsed {
                Some(msg) => {
                    // println!("parsed message: {:?}", msg);
                    // self.order_book.
                    // print_book_event(&msg);
                    // let update = from_kraken(&event);
                    let mut updates = Vec::<PriceLevel>::new();
                    for book_event in msg.data {
                        for level_data in book_event.bids {
                            updates.push(PriceLevel::new(level_data.price, level_data.qty, BID));
                        }
                        for level_data in book_event.asks {
                            updates.push(PriceLevel::new(level_data.price, level_data.qty, ASK));
                        }
                    }
                    order_book.update(&updates);
                    // Clone the order book and send the clone to the logging task
                    let display = DisplayMessage{
                        correlation_id,
                        payload: (order_book.clone(), OrderBookUpdate::new(updates))};
                    if display_tx.send(display).await.is_err() {
                        error!("Logger task has been terminated");
                        break;
                    }
                },
                None => {
                    if  !message.payload().eq("{\"channel\":\"heartbeat\"}") {
                        error!("error while parsing", )
                    } else { () }

                },
                // None => (),
            }
            // println!();





            // dbg!(&event);

        }
    });




    use websocket::models::kraken::translate::from_kraken;

    client
        .send(SubscribeBookRequest::symbol("BTC/USD").depth(Depth::D10))
        .await
        .expect("cannot send request");

    client.start_book_delta(data_tx.clone()).await;

    // while let Some(event) = client.book_delta_events().next().await {
    //
    //     let event: &BookEvent = &event;
    //     // let data = &event.data;
    //     print_book_event(&event);
    //     // let update = from_kraken(&event);
    //     order_book.update_from_kraken(&event.data);
    //
    //
    //     // Clone the order book and send the clone to the logging task
    //     let order_book_clone = order_book.clone();
    //     if tx.send(order_book_clone).await.is_err() {
    //         eprintln!("Logger task has been terminated");
    //         break;
    //     }
    //     // dbg!(&event);
    // }
}
// The function to print BookEvent data
pub fn print_book_event(book_event: &BookEvent) {
    for book_data in &book_event.data {
        println!("Symbol: {}", book_data.symbol);
        println!("Checksum: {}", book_data.checksum);
        println!("Bids:");
        for bid in &book_data.bids {
            println!("\tPrice: {:.2}, Quantity: {:.10}", bid.price, bid.qty);
        }
        println!("Asks:");
        for ask in &book_data.asks {
            println!("\tPrice: {:.2}, Quantity: {:.10}", ask.price, ask.qty);
        }
    }
}