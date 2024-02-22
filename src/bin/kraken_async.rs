use websocket::log_messages::log_messages;
use websocket::connect_and_listen::connect_and_listen_kraken;
use websocket::models::kraken::book::parse;

use tokio::sync::mpsc;
use websocket::messages::IncomingMsg;
use websocket::models::kraken::book::OrderBook;

pub async fn log_order_book(mut receiver: mpsc::Receiver<IncomingMsg>, order_book: &OrderBook) {
    while let Some(msg) = receiver.recv().await {
        match parse::parse_message(&msg.msg) {
            Ok(parsed) => {
                // If parsing succeeds, print the parsed data
                // Adjust the println! to match how you want to display the parsed information
                println!("{:?}", parsed);
            },
            Err(_) => {
                // If parsing fails, print the original message
                println!("Original message: {}", msg.msg);
            },
        }
    }
}

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel(32);

    // Launch the WebSocket listener
    let kraken_handle = {
        let tx = tx.clone();
        tokio::spawn(async move { connect_and_listen_kraken(tx).await; })
    };

    let order_book = OrderBook::new();
    // Launch the logger
    let logger_handle = {
        tokio::spawn(async move { log_order_book(rx, &order_book).await; })
    };

    // Simulate some operation or wait for a condition to trigger shutdown
    // For demonstration, you can wait for the channel to be dropped
    // rx.await;

    // Wait for the WebSocket listener and logger to complete
    kraken_handle.await.expect("Kraken thread panicked");
    logger_handle.await.expect("Logger thread panicked");
}
