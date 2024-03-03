use futures::SinkExt;
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tungstenite::Message;
use url::Url;


// use std::sync::{Arc, Mutex};
// use std::thread;
// use std::time::Duration;
// use tokio::net::TcpStream;
use futures::{StreamExt};
// use tokio::runtime::Runtime;

// use serde_json::{Value, Result};

use crate::messages::IncomingMsg;
use crate::quote::Exchange;



pub async fn connect_and_listen_kraken(sender: mpsc::Sender<IncomingMsg>) {
    let url = Url::parse("wss://ws.kraken.com").unwrap();
    let (mut ws_stream, _) =
        connect_async(&url).await
            .expect("Failed to connect to Kraken");
    println!("connected to Kraken on {}", url);

    let subscribe_message = serde_json::json!({
        "event": "subscribe",
        "pair": ["XBT/USD"],
        "subscription": {
            "name": "book",
            "depth": 10,
        }
    }).to_string();

    ws_stream.send(Message::Text(subscribe_message)).await.
        expect("Failed to subscribe to Kraken");

    while let Some(message) = ws_stream.next().await {
        match message {
            Ok(Message::Text(text)) => {
                println!("{}", text);
                if sender.send(IncomingMsg{exchange: Exchange::Kraken, msg: text}).await.is_err() {
                    eprintln!("Failed to send message from Kraken");
                    break;
                }
            }
            Ok(_) => {
                // Ignore non-Text messages
            }
            Err(err) => {
                eprintln!("Error receiving message from Kraken: {:?}", err);
                break;
            }
        }
    }
}



pub async fn connect_and_listen_binance(sender: mpsc::Sender<IncomingMsg>) {
    let url = Url::parse("wss://stream.binance.com:9443/ws/btcusdt@depth5@100ms").unwrap();
    let (mut ws_stream, _) = connect_async(&url).await.expect("Failed to connect to Binance");
    println!("connected to Binance on {}", url);
    while let Some(message) = ws_stream.next().await {
        match message {
            Ok(Message::Text(text)) => {
                if sender.send(IncomingMsg{exchange: Exchange::Binance, msg: text}).await.is_err() {
                    eprintln!("Failed to send message from Binance");
                    break;
                }
            }
            Ok(_) => {
                // Ignore non-Text messages
            }
            Err(err) => {
                eprintln!("Error receiving message from Binance: {:?}", err);
                break;
            }
        }
    }
}