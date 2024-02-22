use futures::StreamExt;
use kraken_ws_client::api::{
    SubscribeTickerRequest, TickerEvent,
    SubscribeBookRequest, BookEvent
};
use kraken_ws_client::types::Depth;
use websocket::models::order_book::OrderBook;


use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let mut client = kraken_ws_client::connect_public()
        .await
        .expect("cannot connect");

    // client
    //     .send(SubscribeTickerRequest::symbol("BTC/USD"))
    //     .await
    //     .expect("cannot send request");
    //
    // while let Some(event) = client.ticker_events().next().await {
    //     let event: &TickerEvent = &event;
    //     let data = &event.data;
    //     dbg!(event);
    // }

    let (tx, mut rx) = mpsc::channel::<OrderBook>(32); // Create a channel with a buffer size of 32.

    // Spawn a task to handle printing.
    tokio::spawn(async move {
        while let Some(order_book) = rx.recv().await {
            order_book.best();
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
        let update = from_kraken(&event);
        order_book.update(&update);


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