use tokio::sync::mpsc;
use crate::messages::IncomingMsg;

pub async fn log_messages(mut receiver: mpsc::Receiver<IncomingMsg>) {
    while let Some(msg) = receiver.recv().await {
        println!("Logged message: {}", msg.msg);
    }
}