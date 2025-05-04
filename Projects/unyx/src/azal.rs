use color_eyre::Result;
use parking_lot::Mutex;
use azalea::prelude::*;

use std::sync::mpsc::Sender;
use std::sync::Arc;
use once_cell::sync::Lazy;

#[derive(Default, Clone, Component)]
pub struct State {}

pub enum ConsoleType {
    Botlog(String),
    ServerMsg(String),
}

// pub async fn start_azalea(
//     address: &str,
//     tx_log: std::sync::mpsc::Sender<ConsoleType>,
// ) -> Result<()> {
//     let account = Account::offline("ItzBtzz");    

//     ClientBuilder::new()
//         .set_handler(handle)
//         .start(account, address)
//         .await
//         .unwrap();
//     Ok(())
// }

// Global variable to store the sender
static TX_LOG: Lazy<Mutex<Option<Sender<ConsoleType>>>> = Lazy::new(|| Mutex::new(None));

async fn handle(bot: Client, event: Event, state: State) -> color_eyre::Result<()> {
    match event {
        Event::Chat(m) => {
            let message = m.message().to_ansi();
            
            // Send to the channel if available
            if let Some(tx) = &*TX_LOG.lock() {
                let _ = tx.send(ConsoleType::ServerMsg(message));
            }
        }
        _ => {}
    }

    Ok(())
}

fn init_handler(tx: Sender<ConsoleType>) {
    *TX_LOG.lock() = Some(tx);
}

pub async fn start_azalea(
    address: &str,
    tx_log: std::sync::mpsc::Sender<ConsoleType>,
) -> Result<()> {
    let account = Account::offline("ItzBtzz");
    
    // Initialize the global sender
    init_handler(tx_log);

    ClientBuilder::new()
        .set_handler(handle)
        .start(account, address)
        .await
        .unwrap();
    Ok(())
}