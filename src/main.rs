


use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::net::TcpStream;
use futures::{StreamExt, SinkExt};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use serde_json::{Value, Result};

use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

use websocket::messages::IncomingMsg;
use websocket::connect_and_listen::{connect_and_listen_kraken, connect_and_listen_binance};
use websocket::quote::Quote;
use websocket::quote::Exchange;

// struct IncomingMsg {
//     exchange: Exchange,
//     msg: String,
// }
//
// async fn connect_and_listen_kraken(sender: mpsc::Sender<IncomingMsg>) {
//     let url = Url::parse("wss://ws.kraken.com").unwrap();
//     let (mut ws_stream, _) = connect_async(url).await.expect("Failed to connect to Kraken");
//
//     let subscribe_message = serde_json::json!({
//         "event": "subscribe",
//         "pair": ["XBT/USD"],
//         "subscription": {"name": "book", "depth": 1}
//     }).to_string();
//
//     ws_stream.send(Message::Text(subscribe_message)).await.expect("Failed to subscribe to Kraken");
//
//     while let Some(message) = ws_stream.next().await {
//         if let Ok(Message::Text(text)) = message {
//             if sender.send(IncomingMsg{exchange: Exchange::Kraken, msg: text}).await.is_err() {
//                 eprintln!("Failed to send message from Kraken");
//                 break;
//             }
//         }
//     }
// }
//
// async fn connect_and_listen_binance(url: Url, sender: mpsc::Sender<IncomingMsg>) {
//     // let url = Url::parse(&url).unwrap();
//     let (mut ws_stream, _) = connect_async(url).await.expect("Failed to connect to Binance");
//
//     // Binance does not require an explicit subscription message for this stream
//
//     while let Some(message) = ws_stream.next().await {
//         if let Ok(Message::Text(text)) = message {
//             if sender.send(IncomingMsg{exchange: Exchange::Binance, msg: text}).await.is_err() {
//                 eprintln!("Failed to send message from Binance");
//                 break;
//             }
//         }
//     }
// }
//
//
// fn log_messages(mut receiver: mpsc::Receiver<IncomingMsg>) {
//     while let message = receiver.blocking_recv() {
//         if let Some(msg) = message {
//             println!("Logged message: {}", msg.msg);
//         }
//     }
// }
//
// async fn process_and_compare_quotes(mut receiver: mpsc::Receiver<IncomingMsg>)  {
//     let mut last_mid_prices: [Option<f64>; 2] = [None, None]; // [Kraken, Binance]
//
//     while let Some(msg) = receiver.blocking_recv() {
//
//         if let Some(quote) = Quote::parse(&msg) {
//             match quote.exchange {
//                 Exchange::Kraken => last_mid_prices[0] = Some((quote.best_bid + quote.best_ask) / 2.0),
//                 Exchange::Binance => last_mid_prices[1] = Some((quote.best_bid + quote.best_ask) / 2.0),
//             }
//         }
//
//         // Compare mid prices if both are available
//         if let (Some(kraken_mid), Some(binance_mid)) = (last_mid_prices[0], last_mid_prices[1]) {
//             println!("Mid Price Difference (Kraken - Binance): {}", kraken_mid - binance_mid);
//         }
//
//
//     }
// }


async fn process_and_compare_quotes(mut receiver: mpsc::Receiver<IncomingMsg>) {
    let mut last_mid_prices: [Option<f64>; 2] = [None, None]; // [Kraken, Binance]

    while let Some(msg) = receiver.recv().await {
        println!("{}", msg.msg);
        if let Some(quote) = Quote::parse(&msg) {
            match quote.exchange {
                Exchange::Kraken => last_mid_prices[0] = Some((quote.best_bid + quote.best_ask) / 2.0),
                Exchange::Binance => last_mid_prices[1] = Some((quote.best_bid + quote.best_ask) / 2.0),
            }
        }

        // Compare mid prices if both are available
        if let (Some(kraken_mid), Some(binance_mid)) = (last_mid_prices[0], last_mid_prices[1]) {
            println!("Mid Price Difference (Kraken - Binance): {}", kraken_mid - binance_mid);
        }
    }
}

fn main() {
    let rt = Arc::new(Mutex::new(Runtime::new().expect("Failed to create Tokio runtime")));
    let (tx1, rx1) = mpsc::channel(32);

    // let (shutdown_tx, shutdown_rx) = mpsc::channel(1);

    // Launch the WebSocket listeners
    let kraken_handle = {
        let rt = rt.clone();
        let tx1 = tx1.clone();
        thread::spawn(move || {
            rt.lock().unwrap().block_on(connect_and_listen_kraken(tx1));
        })
    };

    let binance_handle = {
        let rt = rt.clone();
        thread::spawn(move || {
            rt.lock().unwrap().block_on(connect_and_listen_binance(tx1));
        })
    };


    // Launch the handler
    let process_and_compare_handle = {
        let rt = rt.clone();
        thread::spawn(move || {
            rt.lock().unwrap().block_on(process_and_compare_quotes(rx1));
        })
    };

    // Simulate some operation or wait for a condition to trigger shutdown
    // thread::sleep(std::time::Duration::from_secs(10)); // For demonstration, wait 10 seconds before shutdown
    // shutdown_tx.blocking_send(()).expect("Failed to send shutdown signal");

    // Wait for the threads to complete
    kraken_handle.join().expect("Kraken thread panicked");
    binance_handle.join().expect("Binance thread panicked");
    process_and_compare_handle.join().expect("Logger thread panicked");
}


