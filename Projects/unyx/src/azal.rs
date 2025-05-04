use color_eyre::Result;
use parking_lot::Mutex;
use azalea::prelude::*;

use crate::SERVER_ADDRESS;

#[derive(Clone, Component)]
pub struct State {
    pub tx_log: std::sync::mpsc::Sender<ConsoleType>,
}

pub enum ConsoleType {
    Botlog(String),
    ServerMsg(String),
}

pub async fn start_azalea(
    address: &str,
    tx_log: std::sync::mpsc::Sender<ConsoleType>,
) -> Result<()> {
    let account = Account::offline("ItzBtzz");

    // Initialize state with the tx_log
    let state = State { tx_log };

    ClientBuilder::new()
        .set_handler(handle)
        .add_plugin(state)  // Add the state as a plugin
        .start(account, address)  // Use the provided address parameter
        .await?;  // Use ? for error propagation
    Ok(())
}

async fn handle(bot: Client, event: Event, state: State) -> color_eyre::Result<()> {
    match event {
        Event::Chat(m) => {
            state.tx_log
                .send(ConsoleType::ServerMsg(format!(
                    "{}",
                    m.message().to_ansi()
                )))
                .unwrap();
        }
        _ => {}
    }

    Ok(())
}