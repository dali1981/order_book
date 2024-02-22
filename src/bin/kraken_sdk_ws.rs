use futures::StreamExt;
use kraken_ws_client::api::{
    SubscribeTickerRequest, TickerEvent,
    SubscribeBookRequest, BookEvent
};
use kraken_ws_client::types::Depth;
use websocket::models::order_book::OrderBook;


use tokio::sync::mpsc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt; // Provides async write functions for File

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_level(true)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .init();

    let mut client = kraken_ws_client::connect_public()
        .await
        .expect("cannot connect");


    let (tx, mut rx) = mpsc::channel::<OrderBook>(32); // Create a channel with a buffer size of 32.

    // Spawn a task to handle printing.
    tokio::spawn(async move {
        // Open or create a file to write. `File::create` will truncate the file if it already exists.
        let mut file = File::create("order_book_log.txt").await.expect("Failed to create file");

        while let Some(order_book) = rx.recv().await {
            let msg = order_book.best();

            // Write to stdout
            println!("{}", &msg);

            // Write to file, appending a newline character for each message
            if let Err(e) = file.write_all(format!("{}\n", &msg).as_bytes()).await {
                eprintln!("Failed to write to file: {}", e);
                // Decide how to handle the write error. For example, you might want to break the loop,
                // or you might simply log the error and continue trying to write new messages.
            }
        }
    });


    let mut order_book:  OrderBook = OrderBook::new();

    use websocket::models::kraken::translate::from_kraken;

    client
        .send(SubscribeBookRequest::symbol("BTC/USD").depth(Depth::D10))
        .await
        .expect("cannot send request");

    while let Some(event) = client.book_delta_events().next().await {

        let event: &BookEvent = &event;
        // let data = &event.data;
        print_book_event(&event);
        // let update = from_kraken(&event);
        order_book.update_from_kraken(&event.data);


        // Clone the order book and send the clone to the logging task
        let order_book_clone = order_book.clone();
        if tx.send(order_book_clone).await.is_err() {
            eprintln!("Logger task has been terminated");
            break;
        }
        // dbg!(&event);
    }
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