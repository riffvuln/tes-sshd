use color_eyre::Result;
use parking_lot::Mutex;
use azalea::prelude::*;

use tokio::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use once_cell::sync::Lazy;

#[derive(Default, Clone, Component)]
pub struct State {}

pub enum ConsoleType {
    Botlog(String),
    ServerMsg(String),
}

pub enum CommandType {
    Chat(String),
    Goto(String),
}

// Global variable to store the sender
static TX_LOG: Lazy<Mutex<Option<Sender<ConsoleType>>>> = Lazy::new(|| Mutex::new(None));
static RX_INPUT: Lazy<Mutex<Option<Arc<Mutex<Receiver<CommandType>>>>>> = Lazy::new(|| Mutex::new(None));

async fn handle(bot: Client, event: Event, state: State) -> color_eyre::Result<()> {
    let rx_option = {
        let rx_input_guard = RX_INPUT.lock();
        rx_input_guard.as_ref().cloned()
    };
    
    match event {
        Event::Login => {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            bot.chat("/login rifaiGG123");
            bot.chat("/register rifaiGG123 rifaiGG123");
        }
        Event::Chat(m) => {
            let message = m.message().to_ansi();
            
            // Send to the channel if available
            if let Some(tx) = &*TX_LOG.lock() {
                let _ = tx.send(ConsoleType::ServerMsg(message)).await;
                // let _ = tx.send(ConsoleType::Botlog("GOT MESSAGE".to_string())).await;
            }
        }
        _ => {}
    }
    
    // Use try_recv() instead of recv() to make it non-blocking
    if let Some(rx_arc) = rx_option {
        let mut rx_input = rx_arc.lock();
        match rx_input.try_recv() {
            Ok(CommandType::Chat(msg)) => {
                bot.chat(&msg);
            }
            Ok(CommandType::Goto(_msg)) => {
                    
            }
            Err(_) => {
                // No message available or channel is disconnected, that's fine
            }
        }
    }
    
    Ok(())
}
fn init_handler(tx: Sender<ConsoleType>, rx: Receiver<CommandType>) {
    *TX_LOG.lock() = Some(tx);
    *RX_INPUT.lock() = Some(Arc::new(Mutex::new(rx)));
}
pub async fn start_azalea(
    address: &str,
    tx_log: tokio::sync::mpsc::Sender<ConsoleType>,
    rx_input: tokio::sync::mpsc::Receiver<CommandType>,
) -> Result<()> {
    let account = Account::offline("ItzBtzz");
    
    // Initialize the global sender
    init_handler(tx_log, rx_input);
        .set_handler(move |bot, event, state| Box::pin(handle(bot, event, state)))
    ClientBuilder::new()
        .set_handler(handle)
        .start(account, address)
        .await
        .unwrap();
    Ok(())
}